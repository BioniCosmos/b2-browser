import { Hono } from 'hono'
import { jsxRenderer } from 'hono/jsx-renderer'
import FileBrowser from './components/FileBrowser'
import { direct } from './services/director'
import { importToRepo } from './services/import'

const app = new Hono<{ Bindings: Env }>()

app.post('/api/import', async (c) => {
  await importToRepo(c.env)
  return c.body(null, 204)
})

app.get(
  '*',
  jsxRenderer(({ children }) => (
    <html>
      <head>
        <meta charset="utf-8" />
        <meta name="viewport" content="width=device-width" />
        <title>B2 Browser</title>
        <link rel="stylesheet" href="/styles.css" />
      </head>
      <body class="bg-gray-100 p-5">{children}</body>
    </html>
  )),
)

app.get('*', async (c) => {
  const result = await direct(c.env, c.req.path)
  if (!result) {
    return c.notFound()
  }
  if (typeof result === 'string') {
    return c.redirect(result)
  }
  return c.render(<FileBrowser file={result} />)
})

export default app
