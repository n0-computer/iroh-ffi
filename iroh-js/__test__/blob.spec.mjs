import test from 'ava'

import { Iroh, SetTagOption } from '../index.js'
import { cwd } from 'process'


test('add blob from path', async (t) => {
  const node = await Iroh.memory()


  // Do not use Promise.withResovlers it is buggy
  let resolve;
  let reject;
  const promise = new Promise((res, rej) => {
    resolve = res
    reject = rej;
  });

  await node.blobs.addFromPath(
    cwd() + '/__test__/index.spec.mjs',
    true,
    SetTagOption.auto(),
    { wrap: false },
    async (error, progress) => {
      if (error != null) {
        return reject(error);
      }

      // console.log('progress', progress)
      if (progress.allDone != null) {
        resolve(progress.allDone)
      }
    }
  )

  const allDone = await promise
  t.is(allDone.format, 'Raw')
  t.truthy(allDone.tag)
  t.truthy(allDone.hash)
})
