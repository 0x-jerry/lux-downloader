import { writeFile } from 'node:fs/promises'

// https://developer.chrome.com/apps/autoupdate#update_manifest

const WEBEXT_APPID = 'infckgbjgkalgpmgfpglleiaodcklcag'

export async function createUpdateYamlFile(opt: { version: string }) {
  const updateContent = `
<?xml version='1.0' encoding='UTF-8'?>
<gupdate xmlns='http://www.google.com/update2/response' protocol='2.0'>
  <app appid='${WEBEXT_APPID}'>
    <updatecheck codebase='https://github.com/0x-jerry/webext-universal/releases/latest/download/webext-universal.crx' version='${opt.version}' status='ok' />
  </app>
</gupdate>
    `.trim()

  await writeFile('updates.xml', updateContent)
}
