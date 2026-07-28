import { browserOnly } from 'next/dist/client/components/browser-only'

export default function Page() {
  browserOnly()
  return <p>deep RSC import should fail</p>
}
