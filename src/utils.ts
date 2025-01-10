export function getParentPath(path: string) {
  return path.split('/').slice(0, -1).join('/') || '/'
}

export function getRedirectPath(currentURL: string | undefined) {
  if (!currentURL) {
    return '/'
  }
  try {
    return new URL(currentURL).searchParams.get('redirect') ?? '/'
  } catch (error) {
    return '/'
  }
}
