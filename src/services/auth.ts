import { getCookie } from 'hono/cookie'
import { createMiddleware } from 'hono/factory'
import { verify } from 'hono/jwt'

export const auth = createMiddleware<{ Bindings: Env }>(async (c, next) => {
  const isLoginPath = c.req.path === '/login'
  const isValidToken = await isValid(
    getCookie(c, 'token') ?? '',
    c.env.JWT_SECRET,
  )
  if (isLoginPath) {
    return isValidToken ? c.redirect('/') : next()
  }
  return isValidToken ? next() : c.redirect('/login')
})

async function isValid(token: string, secret: string) {
  return verify(token, secret)
    .then(() => true)
    .catch(() => false)
}
