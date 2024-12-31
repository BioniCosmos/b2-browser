import { FileRepo } from '../repos/file'

export async function direct(env: Env, path: string) {
  const repo = new FileRepo(env.DB)
  const file = await repo.query(trimEndSlash(path))
  if (!file) {
    return null
  }
  return file.type === 'directory'
    ? file
    : env.DOWNLOAD_BASE_URL +
        file.path.split('/').map(encodeURIComponent).join('/')
}

function trimEndSlash(path: string) {
  return path !== '/' && path.endsWith('/') ? path.slice(0, -1) : path
}
