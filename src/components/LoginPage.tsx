export default function LoginPage() {
  return (
    <div class="min-h-[calc(100vh-1.25rem*2)] flex items-center justify-center bg-gray-100">
      <div class="max-w-md w-full p-8 bg-white rounded-lg shadow">
        <h2 class="mb-8 text-center text-3xl font-extrabold text-gray-900">
          Sign in to your account
        </h2>
        <form
          class="space-y-4"
          hx-post="/login"
          hx-target="#error-message"
          hx-swap="outerHTML"
        >
          <div>
            <label for="username" class="sr-only">
              Username
            </label>
            <input
              id="username"
              name="username"
              type="text"
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-t-md placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              placeholder="Username"
            />
          </div>
          <div>
            <label for="password" class="sr-only">
              Password
            </label>
            <input
              id="password"
              name="password"
              type="password"
              required
              class="w-full px-3 py-2 border border-gray-300 rounded-b-md placeholder-gray-500 text-gray-900 focus:outline-none focus:ring-1 focus:ring-blue-500 focus:border-blue-500 sm:text-sm"
              placeholder="Password"
            />
          </div>
          <button class="w-full py-2 px-4 mt-6 text-sm font-medium text-white bg-blue-600 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500">
            Sign in
          </button>
          <div id="error-message" class="hidden" />
        </form>
      </div>
    </div>
  )
}
