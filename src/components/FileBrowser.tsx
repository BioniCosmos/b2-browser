import { Child } from 'hono/jsx'
import type { File } from '../types/file'
import { getParentPath } from '../utils'

export interface FileBrowserProps {
  file: File
}

export default function FileBrowser({ file }: FileBrowserProps) {
  const breadcrumbs = file.path
    .split('/')
    .filter(Boolean)
    .flatMap((part, i, pathParts) => [
      <span> / </span>,
      <BreadcrumbItem name={part} path={getCurrentPath(pathParts, i)} />,
    ])
  const filterFileType = (fileType: File['type']) =>
    file.children!.filter(({ type }) => type === fileType).toSorted()
  return (
    <div class="bg-white rounded-lg shadow-md p-5 max-w-7xl mx-auto">
      <div class="mb-5 p-3 bg-gray-50 rounded font-bold text-2xl">
        <BreadcrumbItem name="root" path="/" />
        {breadcrumbs}
      </div>
      <table class="w-full border-collapse">
        <thead>
          <tr>
            <TableHeaderCell>Name</TableHeaderCell>
            <TableHeaderCell>Size</TableHeaderCell>
            <TableHeaderCell>Last Modified</TableHeaderCell>
          </tr>
        </thead>
        <tbody>
          {file.path !== '/' && (
            <TableRow>
              <FileNameCell
                name="../ (Parent Directory)"
                path={getParentPath(file.path)}
              />
              <TableCell>-</TableCell>
              <TableCell>-</TableCell>
            </TableRow>
          )}
          {filterFileType('directory').map(({ name, path }) => (
            <TableRow key={path}>
              <FileNameCell name={`📁  ${name}`} path={path} />
              <TableCell>-</TableCell>
              <TableCell>-</TableCell>
            </TableRow>
          ))}
          {filterFileType('file').map(({ name, path, size, lastModified }) => (
            <TableRow key={path}>
              <FileNameCell name={`📄  ${name}`} path={path} />
              <TableCell>{formatSize(size!)}</TableCell>
              <TableCell>{formatDate(lastModified!)}</TableCell>
            </TableRow>
          ))}
        </tbody>
      </table>
    </div>
  )
}

function BreadcrumbItem({ name, path }: { name: string; path: string }) {
  return (
    <a href={path} class="hover:underline">
      {name}
    </a>
  )
}

function TableHeaderCell({ children }: { children: Child }) {
  return (
    <th class="p-3 text-left bg-gray-50 font-semibold text-gray-600">
      {children}
    </th>
  )
}

function TableRow({ children, key }: { children: Child; key?: string }) {
  return (
    <tr class="hover:bg-gray-50" key={key}>
      {children}
    </tr>
  )
}
function FileNameCell({ name, path }: { name: string; path: string }) {
  return (
    <TableCell>
      <a href={path} class="text-blue-600 hover:underline block">
        {name}
      </a>
    </TableCell>
  )
}

function TableCell({ children }: { children: Child }) {
  return <td class="p-3 border-b border-gray-100">{children}</td>
}

function getCurrentPath(pathParts: string[], i: number) {
  return `/${pathParts.slice(0, i + 1).join('/')}`
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
