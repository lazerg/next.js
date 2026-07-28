import { browserOnly } from 'next/navigation'

export const dynamic = 'force-dynamic'

export default function Page() {
  browserOnly()
  return <p>dynamic RSC import should fail during compilation</p>
}
