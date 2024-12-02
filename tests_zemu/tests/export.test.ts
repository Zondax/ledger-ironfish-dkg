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

import { DEFAULT_START_OPTIONS, isTouchDevice, zondaxMainmenuNavigation } from '@zondax/zemu'
import { TModel } from '@zondax/zemu/dist/types'
import { buildTx } from './utils'
import { defaultOptions, identities, models, restoreKeysTestCases } from './common'

// This is required for stax and flex as the ledger label is different to the one Zondax has
// This app enables (ledger) vs This application enables (zondax)
const startTextFn = (model: TModel) => (isTouchDevice(model) ? 'Ironfish DKG' : DEFAULT_START_OPTIONS.startText)

describe('Basic', function () {
  test.each(models)('export tx', async function (m) {
    try {
      console.log('export')
      const unsignedTxRaw = buildTx(
        restoreKeysTestCases[0].publicAddress,
        restoreKeysTestCases[0].viewKeys,
        restoreKeysTestCases[0].proofKeys,
      )
      console.log('unsignedTxRaw ' + unsignedTxRaw.toString('hex'))
    } finally {
    }
  })
})
