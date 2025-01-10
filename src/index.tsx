import { Hono } from 'hono'
import { setCookie } from 'hono/cookie'
import { jsxRenderer } from 'hono/jsx-renderer'
import FileBrowser from './components/FileBrowser'
import LoginPage from './components/LoginPage'
import { auth } from './services/auth'
import { direct } from './services/director'
import { importToRepo } from './services/import'
import { login } from './services/login'
import { getRedirectPath } from './utils'

const app = new Hono<{ Bindings: Env }>()

app.use('*', auth)

app.post('/login', async (c) => {
  const { username, password } = await c.req.parseBody()
  const [success, result] = await login(c.env, username, password)
  if (!success) {
    return c.html(result)
  }
  setCookie(c, 'token', result)
  return c.body(null, 204, {
    'HX-Location': getRedirectPath(c.req.header('HX-Current-URL')),
  })
})

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
      <body class="bg-gray-100 p-5">
        {children}
        <script src="/htmx.min.js" />
      </body>
    </html>
  )),
)

app.get('/login', async (c) => c.render(<LoginPage />))

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
