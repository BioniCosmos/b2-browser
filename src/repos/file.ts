import { PrismaD1 } from '@prisma/adapter-d1'
import { PrismaClient, type Prisma, type File as RawFile } from '@prisma/client'
import type { ITXClientDenyList } from '@prisma/client/runtime/library'
import type { File } from '../types/file'
import { getParentPath } from '../utils'

export class FileRepo {
  private client: PrismaClient

  constructor(dbSource: D1Database) {
    this.client = new PrismaClient({ adapter: new PrismaD1(dbSource) })
  }

  async query(path: string) {
    const rawFile = await this.client.file.findUnique({
      where: { path },
      include: { children: true },
    })
    return rawFile ? toFile(rawFile) : null
  }

  async create(file: Omit<File, 'children'>) {
    if (await this.query(file.path)) {
      return
    }
    const parent = await this.ensureHierarchies(this.client, file.path)
    await this.client.file.create({
      data: { ...file, parentId: parent.id },
    })
  }

  private async ensureHierarchies(tx: Tx, path: string): Promise<RawFile> {
    const parentPath = getParentPath(path)
    const parent = await tx.file.findUnique({
      where: { path: parentPath },
    })
    if (parent) {
      return parent
    }
    if (parentPath === '/') {
      const [file] = await tx.$queryRaw<
        RawFile[]
      >`INSERT INTO "File" (type, name, path) VALUES ('directory', '/', '/') ON CONFLICT (path) DO UPDATE SET id = "File".id RETURNING *`
      return file
    }
    const grandParent = await this.ensureHierarchies(tx, parentPath)
    const [file] = await tx.$queryRaw<
      RawFile[]
    >`INSERT INTO "File" (type, name, path, "parentId") VALUES ('directory', ${getParentName(
      path,
    )}, ${parentPath}, ${
      grandParent.id
    }) ON CONFLICT (path) DO UPDATE SET id = "File".id RETURNING *`
    return file
  }

  async deleteAll() {
    await this.client.file.deleteMany()
  }
}

function getParentName(path: string) {
  return path.split('/').at(-2) || '/'
}

function toFile({
  type,
  name,
  path,
  size,
  contentType,
  lastModified,
  ...rest
}: RawFile | RawParentFile): File {
  return {
    type: type as File['type'],
    name,
    path,
    children: 'children' in rest ? rest.children.map(toFile) : undefined,
    size: Number(size) ?? undefined,
    contentType: contentType ?? undefined,
    lastModified: Number(lastModified) ?? undefined,
  }
}

type Tx = Omit<PrismaClient, ITXClientDenyList>
type RawParentFile = Prisma.FileGetPayload<{ include: { children: true } }>
