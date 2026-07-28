import { join } from 'path'
import type { Server } from 'net'
import { isReact18, nextTestSetup } from 'e2e-utils'
import { renderViaHTTP, startCleanStaticServer } from 'next-test-utils'

describe('browserOnly with static export', () => {
  const { next, isNextDev, skipped } = nextTestSetup({
    files: __dirname,
    nextConfig: {
      output: 'export',
    },
    skipStart: true,
    skipDeployment: true,
  })

  if (skipped) {
    return
  }

  if (isNextDev || process.env.__NEXT_CACHE_COMPONENTS === 'true') {
    it.skip('requires an export-compatible production build', () => {})
    return
  }

  let server: Server
  let port: number

  beforeAll(async () => {
    const result = await next.build()
    expect(result.exitCode).toBe(0)

    server = await startCleanStaticServer(join(next.testDir, 'out'))
    const address = server.address()
    if (address === null || typeof address === 'string') {
      throw new Error('Expected the static export server to listen on a port')
    }
    port = address.port
  })

  afterAll(async () => {
    if (server) {
      await new Promise<void>((resolve) => server.close(() => resolve()))
    }
  })

  it('exports the server fallback and hydrates browser-only content', async () => {
    const html = await renderViaHTTP(port, '/')
    expect(html).toContain('static fallback')
    expect(html.includes('id="browser-content"')).toBe(false)

    const browser = await next.browser('/', { baseUrl: port })
    expect(await browser.elementByCss('#browser-content').text()).toBe(
      'static browser content'
    )
  })
  ;(isReact18 ? it.skip : it)(
    'exports a Pages Router fallback without reporting bailout errors',
    async () => {
      const html = await renderViaHTTP(port, '/browser-only')
      expect(html).toContain('pages server sibling')
      expect(html).toContain('pages fallback')
      expect(html.includes('id="pages-browser-content"')).toBe(false)

      const browser = await next.browser('/browser-only', {
        baseUrl: port,
        pushErrorAsConsoleLog: true,
      })
      expect(await browser.elementByCss('#pages-browser-content').text()).toBe(
        'pages browser content'
      )

      const logs = await browser.log()
      expect(logs.filter((entry) => entry.source === 'error')).toEqual([])
      expect(
        next.cliOutput.includes(
          'Bail out to client-side rendering: browserOnly()'
        )
      ).toBe(false)
    }
  )
})
