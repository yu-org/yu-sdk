/**
 * Integration test for the JavaScript yu SDK.
 *
 * Prerequisites: a yu chain with poa+asset tripods running on localhost.
 *   cd ../../testchain && go run main.go
 *
 * Run:
 *   cd js && npm install && npm test
 */

import { YuClient, KeyPair } from "../src/index.js";

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

async function queryBalance(client, address) {
  const resp = await client.readChain("asset", "QueryBalance", { account: address });
  return Number(resp.amount ?? 0);
}

test("create account and transfer asset", async () => {
  const kp = new KeyPair();
  const toKp = new KeyPair();

  const client = new YuClient("http://localhost:7999", "ws://localhost:8999");
  client.withKeypair(kp);

  const myAddr = await kp.getAddress();
  const toAddr = await toKp.getAddress();

  const events = [];
  const sub = client.subscribeEvents((r) => events.push(r));

  const CREATE_AMOUNT = 500;
  const TRANSFER_1 = 50;
  const TRANSFER_2 = 100;

  // CreateAccount
  await client.writeChain("asset", "CreateAccount", { amount: CREATE_AMOUNT });
  await sleep(10000);
  expect(await queryBalance(client, myAddr)).toBe(CREATE_AMOUNT);

  // Transfer 1
  await client.writeChain("asset", "Transfer", { to: toAddr, amount: TRANSFER_1 });
  await sleep(8000);
  expect(await queryBalance(client, myAddr)).toBe(CREATE_AMOUNT - TRANSFER_1);
  expect(await queryBalance(client, toAddr)).toBe(TRANSFER_1);

  // Transfer 2
  await client.writeChain("asset", "Transfer", { to: toAddr, amount: TRANSFER_2 });
  await sleep(6000);
  expect(await queryBalance(client, myAddr)).toBe(CREATE_AMOUNT - TRANSFER_1 - TRANSFER_2);
  expect(await queryBalance(client, toAddr)).toBe(TRANSFER_1 + TRANSFER_2);

  expect(events.length).toBeGreaterThanOrEqual(1);
  sub.close();
}, 60000);
