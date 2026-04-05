import { writeFile } from 'node:fs/promises'

const PRIVATE_KEY = process.env.WEBEXT_PRIVATE_KEY

if (PRIVATE_KEY) {
  await writeFile('key.pem', PRIVATE_KEY)
}
