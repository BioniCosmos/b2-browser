export interface File {
  type: 'file' | 'directory'
  name: string
  path: string
  children?: Record<string, File>
  size?: number
  contentType?: string
  lastModified?: number
}
