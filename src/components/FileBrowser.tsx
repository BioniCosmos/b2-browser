import type { File } from '../types/file'
import { getParentPath } from '../utils'

export interface FileBrowserProps {
  file: File
}

export default function FileBrowser({ file }: FileBrowserProps) {
  const getCurrentPath = (pathParts: string[], i: number) =>
    pathParts.slice(0, i + 1).join('/')
  const breadcrumbs = file.path
    .split('/')
    .filter(Boolean)
    .flatMap((part, i, pathParts) => [
      <span> / </span>,
      <a href={getCurrentPath(pathParts, i)}>{part}</a>,
    ])

  const directories = file
    .children!.filter(({ type }) => type === 'directory')
    .toSorted()
  const files = file.children!.filter(({ type }) => type === 'file').toSorted()

  return (
    <div class="file-browser">
      <div class="breadcrumb">
        <a href="/">root</a>
        {breadcrumbs}
      </div>
      <table class="file-list">
        <thead>
          <tr>
            <th>Name</th>
            <th>Size</th>
            <th>Last Modified</th>
          </tr>
        </thead>
        <tbody>
          {file.path !== '/' && (
            <tr>
              <td>
                <a href={getParentPath(file.path)}>../ (Parent Directory)</a>
              </td>
              <td>-</td>
              <td>-</td>
            </tr>
          )}
          {directories.map(({ name, path }) => (
            <tr key={name}>
              <td>
                <a href={path}>📁 {name}</a>
              </td>
              <td>-</td>
              <td>-</td>
            </tr>
          ))}
          {files.map(({ name, path, size, lastModified }) => (
            <tr key={name}>
              <td>
                <a href={path}>📄 {name}</a>
              </td>
              <td>{formatSize(size!)}</td>
              <td>{formatDate(lastModified!)}</td>
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function formatSize(size: number) {
  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  let value = size
  let unitIndex = 0

  while (value >= 1024 && unitIndex < units.length - 1) {
    value /= 1024
    unitIndex++
  }

  return `${value.toFixed(1)} ${units[unitIndex]}`
}

function formatDate(timestamp: number) {
  return new Date(timestamp).toLocaleString()
}
