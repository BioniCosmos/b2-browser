export interface File {
  type: 'file' | 'directory'
  name: string
  path: string
  size: number
  contentType: string
  children: Record<string, File>
  lastModified?: number
}

export class FileRepo {
  constructor(private readonly repo: KVNamespace) {}

  async query(path: string): Promise<File | null> {
    const node = await this.repo.get(path)
    return node ? JSON.parse(node) : null
  }

  async create(file: File) {
    const parent = await this.query(this.getParentPath(file.path))
    if (parent) {
      parent.children[file.name] = file
      await this.repo.put(parent.path, JSON.stringify(parent))
    }
    return this.repo.put(file.path, JSON.stringify(file))
  }

  private getParentPath(path: string) {
    return `/${path.split('/').slice(0, -1).join('/')}`
  }
}
