import { nextTestSetup } from 'e2e-utils'

describe('browserOnly deep-imported by an RSC', () => {
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

  it('is poisoned by client-only', async () => {
    const result = await next.build()

    expect(result.exitCode).toBe(1)
    expect(result.cliOutput).toContain('client-only')
    expect(result.cliOutput).toContain('Server Component')
  })
})
