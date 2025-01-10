import type { JSX } from 'hono/jsx/jsx-runtime'
import { sign } from 'hono/jwt'

export async function login(
  env: Env,
  username: string | File,
  password: string | File,
): Promise<[true, string] | [false, JSX.Element]> {
  if (username !== env.USERNAME || password !== env.PASSWORD) {
    return [
      false,
      <div id="error-message" class="text-red-500 text-sm text-center">
        Invalid username or password
      </div>,
    ]
  }
  return [true, await sign({ username }, env.JWT_SECRET)]
}
