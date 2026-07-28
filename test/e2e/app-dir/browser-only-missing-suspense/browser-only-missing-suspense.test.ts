import { nextTestSetup } from 'e2e-utils'

describe('browserOnly without Suspense', () => {
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

  it('fails the build', async () => {
    const result = await next.build()

    expect(result.exitCode).toBe(1)
    expect(result.cliOutput).toContain(
      'browserOnly() should be wrapped in a suspense boundary'
    )
  })
})
