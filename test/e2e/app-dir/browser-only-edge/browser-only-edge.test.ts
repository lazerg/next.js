import { nextTestSetup } from 'e2e-utils'

describe('browserOnly in the Edge runtime', () => {
  const { next, skipped } = nextTestSetup({
    files: __dirname,
    skipStart: true,
    skipDeployment: true,
  })

  if (skipped) {
    return
  }

  if (process.env.__NEXT_CACHE_COMPONENTS === 'true') {
    it.skip('the Edge runtime is incompatible with Cache Components', () => {})
    return
  }

  beforeAll(async () => {
    await next.start()
  })

  it('renders a fallback on the server and content after hydration', async () => {
    const $ = await next.render$('/')
    expect($('#edge-fallback').text()).toBe('edge fallback')
    expect($('#edge-browser-content').length).toBe(0)

    const browser = await next.browser('/')
    expect(await browser.elementByCss('#edge-browser-content').text()).toBe(
      'edge browser content'
    )
  })
})
