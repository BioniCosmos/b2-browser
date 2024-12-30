import {
  authorizeAccount,
  listFileNames,
  type File as B2File,
} from '../lib/b2-api'
import { FileRepo } from '../repos/file'

export async function importToRepo(env: Env) {
  const { apiInfo, authorizationToken } = await authorizeAccount(
    env.B2_APP_ID,
    env.B2_APP_KEY,
  )
  async function getAllFiles(startFileName?: string): Promise<B2File[]> {
    const { files, nextFileName } = await listFileNames(
      { apiBaseURL: apiInfo.storageApi.apiUrl, token: authorizationToken },
      env.B2_BUCKET_ID,
      startFileName,
    )
    if (!nextFileName) {
      return files
    }
    return files.concat(await getAllFiles(nextFileName))
  }
  const files = await getAllFiles()
  const repo = new FileRepo(env.DB)
  await repo.deleteAll()
  await Promise.all(
    files.map(({ contentLength, contentType, fileName, uploadTimestamp }) =>
      repo.create({
        type: 'file',
        name: getFileName(fileName),
        path: `/${fileName}`,
        size: contentLength,
        contentType,
        lastModified: uploadTimestamp,
      }),
    ),
  )
}

function getFileName(path: string) {
  return path.split('/').at(-1) || '/'
}
