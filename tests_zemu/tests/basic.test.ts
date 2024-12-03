/** ******************************************************************************
 *  (c) 2018 - 2024 Zondax AG
 *
 *  Licensed under the Apache License, Version 2.0 (the "License");
 *  you may not use this file except in compliance with the License.
 *  You may obtain a copy of the License at
 *
 *      http://www.apache.org/licenses/LICENSE-2.0
 *
 *  Unless required by applicable law or agreed to in writing, software
 *  distributed under the License is distributed on an "AS IS" BASIS,
 *  WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 *  See the License for the specific language governing permissions and
 *  limitations under the License.
 ******************************************************************************* */

import Zemu, { DEFAULT_START_OPTIONS, isTouchDevice, zondaxMainmenuNavigation } from '@zondax/zemu'
import { defaultOptions, models } from './common'
import IronfishApp from '@zondax/ledger-ironfish'
import { TModel } from '@zondax/zemu/dist/types'

jest.setTimeout(60000)

// This is required for stax and flex as the ledger label is different to the one Zondax has
// This app enables (ledger) vs This application enables (zondax)
const startTextFn = (model: TModel) => (isTouchDevice(model) ? 'Ironfish DKG' : DEFAULT_START_OPTIONS.startText)

describe('Basic', function () {
  test.each(models)('can start and stop container', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name, startText: startTextFn(m.name) })
    } finally {
      await sim.close()
    }
  })

  // TODO fix ST and FL main menu
  test.each(models.filter(v => v.name === 'nanosp'))('main menu', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name, startText: startTextFn(m.name) })
      const nav = !isTouchDevice(m.name) ? zondaxMainmenuNavigation(m.name, [4, -4]) : zondaxMainmenuNavigation(m.name, [1, -1])
      await sim.navigateAndCompareSnapshots('.', `${m.prefix.toLowerCase()}-mainmenu`, nav.schedule)
    } finally {
      await sim.close()
    }
  })

  test.each(models)('get app version', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name, startText: startTextFn(m.name) })
      const app = new IronfishApp(sim.getTransport(), true)

      const resp = await app.getVersion()
      console.log(resp)

      expect(resp.testMode).toBe(false)
      expect(resp.major).toBe(1)
      expect(resp.minor).toBe(1)
      expect(resp.patch).toBe(2)
    } finally {
      await sim.close()
    }
  })

  test.each(models)('get app info', async function (m) {
    const sim = new Zemu(m.path)
    try {
      await sim.start({ ...defaultOptions, model: m.name, startText: startTextFn(m.name) })
      const app = new IronfishApp(sim.getTransport(), true)

      const resp = await app.appInfo()
      console.log(resp)
    } finally {
      await sim.close()
    }
  })
})
