import { defineConfig } from '@0x-jerry/x-release'
import { createUpdateYamlFile } from './scripts/createUpdateFile'

export default defineConfig({
  async beforeCommit(ctx) {
    await createUpdateYamlFile({ version: ctx.nextVersion })
  },
})
