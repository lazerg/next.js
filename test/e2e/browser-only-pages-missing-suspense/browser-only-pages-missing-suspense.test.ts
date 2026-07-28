import { isReact18, nextTestSetup } from 'e2e-utils'
;(isReact18 ? describe.skip : describe)(
  'browser-only-pages-missing-suspense',
  () => {
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
  }
)
