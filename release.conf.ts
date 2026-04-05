import { defineConfig } from '@0x-jerry/x-release'
import { createUpdateYamlFile } from './scripts/createUpdateFile'

export default defineConfig({
  async beforeCommit(ctx) {
    const extensionPkgFile = 'extension/package.json'
    const pkg = await Bun.file(extensionPkgFile).json()
    pkg.version = ctx.nextVersion

    await Bun.write(extensionPkgFile, JSON.stringify(pkg, null, 2))

    const updateContent = await createUpdateYamlFile({ version: ctx.nextVersion })

    await Bun.write('extension/updates.xml', updateContent)
  },
})
