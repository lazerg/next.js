import { nextTestSetup } from 'e2e-utils'

describe('browserOnly imported by an RSC', () => {
  const { next, isNextDev, skipped } = nextTestSetup({
    files: __dirname,
    skipStart: true,
    skipDeployment: true,
  })

  if (skipped) {
    return
  }

  if (isNextDev) {
    it.skip('requires a production build', () => {})
    return
  }

  it('rejects browserOnly in a dynamic RSC during compilation', async () => {
    const result = await next.build()

    expect(result.exitCode).toBe(1)
    expect(result.cliOutput).toContain(
      "You're importing a module that depends on `browserOnly` into a React Server Component module."
    )
    expect(result.cliOutput).toContain(
      'This API is only available in Client Components.'
    )
  })
})
