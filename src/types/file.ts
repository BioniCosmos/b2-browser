export interface File {
  type: 'file' | 'directory'
  name: string
  path: string
  children?: File[]
  size?: number
  contentType?: string
  lastModified?: number
}
