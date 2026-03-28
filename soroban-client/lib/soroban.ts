import { env } from "./env";

import { Horizon, TransactionBuilder, Operation, Networks } from "@stellar/stellar-sdk";
import { nativeToScVal } from "@stellar/stellar-base";
import { signTransaction } from "@stellar/freighter-api";
const { Server } = Horizon;
import StellarSdk from "@stellar/stellar-sdk";
import { nativeToScVal, scValToNative } from "@stellar/stellar-base";

const { Server, TransactionBuilder, Operation, Networks } = StellarSdk;

// Configuration helpers – prefer environment variables so they can be swapped
// for different networks (testnet / preview / mainnet) without changing code.
const HORIZON_URL = env.NEXT_PUBLIC_HORIZON_URL;
const SOROBAN_RPC_URL = env.NEXT_PUBLIC_SOROBAN_RPC_URL;
const NETWORK_PASSPHRASE = env.NEXT_PUBLIC_NETWORK_PASSPHRASE;
const EVENT_MANAGER_CONTRACT = env.NEXT_PUBLIC_EVENT_MANAGER_CONTRACT;

export interface CreateEventParams {
  organizer: string;
  theme: string;
  eventType: string;
  startTimeUnix: number;
  endTimeUnix: number;
  ticketPrice: bigint;
  totalTickets: bigint;
  paymentToken: string;
}

export type SignTransactionFn = (
  txXdr: string,
  options: { networkPassphrase: string; address: string }
) => Promise<string>;

export interface PurchaseTicketParams {
  buyer: string;
  eventId: number;
}

export interface PurchaseTicketsParams {
  buyer: string;
  eventId: number;
  quantity: bigint;
}

export interface SorobanSubmitResult {
  hash: string;
  ledger: number;
  status: string;
}

export interface Event {
  id: number;
  theme: string;
  organizer: string;
  event_type: string;
  total_tickets: bigint;
  tickets_sold: bigint;
  ticket_price: bigint;
  start_date: number;
  end_date: number;
  is_canceled: boolean;
  ticket_nft_addr: string;
  payment_token: string;
}

export interface UpdateEventParams {
  organizer: string;
  event_id: number;
  theme?: string;
  ticket_price?: bigint;
  total_tickets?: bigint;
  start_date?: number;
  end_date?: number;
}

export function isEventManagerConfigured() {
  return EVENT_MANAGER_CONTRACT !== "<MISSING_CONTRACT_ID>";
}

export function getTxExplorerUrl(txHash: string): string {
  const testnet = NETWORK_PASSPHRASE === Networks.TESTNET;
  const base = testnet
    ? "https://stellar.expert/explorer/testnet/tx/"
    : "https://stellar.expert/explorer/public/tx/";
  return `${base}${txHash}`;
}

function resolveSimulationSource(explicit?: string | null): string | null {
  if (explicit) return explicit;
  const env = process.env.NEXT_PUBLIC_SOROBAN_SIM_SOURCE;
  return env && env.length > 0 ? env : null;
}

async function waitForRpcTransaction(
  rpcServer: InstanceType<typeof rpc.Server>,
  hash: string,
  maxAttempts = 40,
  delayMs = 1500
): Promise<rpc.Api.GetSuccessfulTransactionResponse> {
  for (let i = 0; i < maxAttempts; i++) {
    const got = await rpcServer.getTransaction(hash);
    if (got.status === rpc.Api.GetTransactionStatus.SUCCESS) {
      return got;
    }
    if (got.status === rpc.Api.GetTransactionStatus.FAILED) {
      throw new Error(`Transaction failed on-chain: ${hash}`);
    }
    await new Promise((r) => setTimeout(r, delayMs));
  }
  throw new Error(`Timed out waiting for transaction ${hash}`);
}

async function simulateSignAndSend(
  publicKey: string,
  functionName: string,
  args: ReturnType<typeof nativeToScVal>[],
  signTransactionFn: SignTransactionFn
): Promise<SorobanSubmitResult> {
  if (!isEventManagerConfigured()) {
    throw new Error(
      "EVENT_MANAGER_CONTRACT is not configured. Set NEXT_PUBLIC_EVENT_MANAGER_CONTRACT in your env."
    );
  }

  const rpcServer = new rpc.Server(SOROBAN_RPC_URL);
  const server = new Server(HORIZON_URL);
  const account = await server.loadAccount(publicKey);
  const fee = await server.fetchBaseFee();

  const operation = Operation.invokeContractFunction({
    contract: EVENT_MANAGER_CONTRACT,
    function: functionName,
    args,
  });

  const tx = new TransactionBuilder(account, {
    fee: fee.toString(),
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(operation)
    .setTimeout(30)
    .build();

  const sim = await rpcServer.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(sim)) {
    const err = sim.error;
    const msg =
      typeof err === "string"
        ? err
        : (err as { message?: string })?.message || JSON.stringify(err);
    throw new Error(`Simulation failed: ${msg}`);
  }

  const prepared = rpc.assembleTransaction(tx, sim).build();
  const signedXdr = await signTransactionFn(prepared.toXDR(), {
    networkPassphrase: NETWORK_PASSPHRASE,
    address: publicKey,
  });

  const signedTx = TransactionBuilder.fromXDR(signedXdr, NETWORK_PASSPHRASE);
  const sent = await rpcServer.sendTransaction(signedTx);

  if (sent.status === "ERROR") {
    throw new Error(`Submit rejected: ${sent.status}`);
  }

  const confirmed = await waitForRpcTransaction(rpcServer, sent.hash);
  return {
    hash: sent.hash,
    ledger: confirmed.ledger,
    status: confirmed.status,
  };
}

/**
 * Build and submit `purchase_ticket` (single ticket) via EventManager + Freighter (or other provider).
 */
export async function purchaseTicket(
  params: PurchaseTicketParams,
  signTransactionFn: SignTransactionFn
): Promise<SorobanSubmitResult> {
  const args = [
    nativeToScVal(params.buyer, { type: "address" }),
    nativeToScVal(params.eventId, { type: "u32" }),
  ];
  return simulateSignAndSend(params.buyer, "purchase_ticket", args, signTransactionFn);
}

/**
 * Build and submit `purchase_tickets` for quantity &gt; 1 in one transaction.
 */
export async function purchaseTickets(
  params: PurchaseTicketsParams,
  signTransactionFn: SignTransactionFn
): Promise<SorobanSubmitResult> {
  if (params.quantity <= BigInt(0)) {
    throw new Error("Quantity must be at least 1.");
  }
  if (params.quantity === BigInt(1)) {
    return purchaseTicket(
      { buyer: params.buyer, eventId: params.eventId },
      signTransactionFn
    );
  }
  const args = [
    nativeToScVal(params.buyer, { type: "address" }),
    nativeToScVal(params.eventId, { type: "u32" }),
    nativeToScVal(params.quantity, { type: "u128" }),
  ];
  return simulateSignAndSend(params.buyer, "purchase_tickets", args, signTransactionFn);
}

export async function createEvent(
  params: CreateEventParams,
  signTransactionFn: SignTransactionFn
): Promise<SorobanSubmitResult> {
  const args = [
    nativeToScVal(params.organizer, { type: "address" }),
    nativeToScVal(params.theme, { type: "string" }),
    nativeToScVal(params.eventType, { type: "string" }),
    nativeToScVal(params.startTimeUnix, { type: "u64" }),
    nativeToScVal(params.endTimeUnix, { type: "u64" }),
    nativeToScVal(params.ticketPrice, { type: "i128" }),
    nativeToScVal(params.totalTickets, { type: "u128" }),
    nativeToScVal(params.paymentToken, { type: "address" }),
  ];
  return simulateSignAndSend(
    params.organizer,
    "create_event",
    args,
    signTransactionFn
  );
}

export async function getAllEvents(simulationSource?: string | null): Promise<Event[]> {
  if (!isEventManagerConfigured()) {
    return [];
  }

  const source = resolveSimulationSource(simulationSource);
  if (!source) {
    return [];
  }

  const rpcServer = new rpc.Server(SOROBAN_RPC_URL);
  const server = new Server(HORIZON_URL);

  const operation = Operation.invokeContractFunction({
    contract: EVENT_MANAGER_CONTRACT,
    function: "get_all_events",
    args: [],
  });

  const account = await server.loadAccount(source);
  const fee = await server.fetchBaseFee();
  const tx = new TransactionBuilder(account, {
    fee: fee.toString(),
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(operation)
    .setTimeout(30)
    .build();

  const simResult = await rpcServer.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(simResult)) {
    const err = simResult.error;
    throw new Error(
      typeof err === "string" ? err : `Simulation failed: ${JSON.stringify(err)}`
    );
  }

  const success = simResult as rpc.Api.SimulateTransactionSuccessResponse;
  const returnVal = success.result?.retval;
  if (!returnVal) return [];

  const raw = scValToNative(returnVal) as Record<string, unknown>[];
  return raw.map((e) => ({
    id: Number(e.id),
    theme: String(e.theme),
    organizer: String(e.organizer),
    event_type: String(e.event_type),
    total_tickets: BigInt(String(e.total_tickets)),
    tickets_sold: BigInt(String(e.tickets_sold)),
    ticket_price: BigInt(String(e.ticket_price)),
    start_date: Number(e.start_date),
    end_date: Number(e.end_date),
    is_canceled: Boolean(e.is_canceled),
    ticket_nft_addr: String(e.ticket_nft_addr),
    payment_token: String(e.payment_token),
  }));
}

export async function cancelEvent(
  organizer: string,
  eventId: number,
  signTransactionFn: SignTransactionFn
) {
  return simulateSignAndSend(
    organizer,
    "cancel_event",
    [nativeToScVal(eventId, { type: "u32" })],
    signTransactionFn
  );
}

export async function updateEvent(
  params: UpdateEventParams,
  signTransactionFn: SignTransactionFn
) {
  const toOption = (val: bigint | number | string | undefined, type: string) =>
    val !== undefined
      ? nativeToScVal({ Some: nativeToScVal(val, { type }) }, { type: "option" })
      : nativeToScVal(null, { type: "option" });

  return simulateSignAndSend(
    params.organizer,
    "update_event",
    [
      nativeToScVal(params.event_id, { type: "u32" }),
      toOption(params.theme, "string"),
      toOption(params.ticket_price, "i128"),
      toOption(params.total_tickets, "u128"),
      toOption(params.start_date, "u64"),
      toOption(params.end_date, "u64"),
    ],
    signTransactionFn
  );
}

export async function claimFunds(
  organizer: string,
  eventId: number,
  signTransactionFn: SignTransactionFn
) {
  return simulateSignAndSend(
    organizer,
    "withdraw_funds",
    [nativeToScVal(eventId, { type: "u32" })],
    signTransactionFn
  );
}

export async function getEventAttendees(
  eventId: number,
  simulationSource?: string | null
): Promise<string[]> {
  if (!isEventManagerConfigured()) {
    return [];
  }

  const source = resolveSimulationSource(simulationSource);
  if (!source) {
    return [];
  }

  const rpcServer = new rpc.Server(SOROBAN_RPC_URL);
  const server = new Server(HORIZON_URL);

  const operation = Operation.invokeContractFunction({
    contract: EVENT_MANAGER_CONTRACT,
    function: "get_event_buyers",
    args: [nativeToScVal(eventId, { type: "u32" })],
  });

  const account = await server.loadAccount(source);
  const fee = await server.fetchBaseFee();
  const tx = new TransactionBuilder(account, {
    fee: fee.toString(),
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(operation)
    .setTimeout(30)
    .build();

  const simResult = await rpcServer.simulateTransaction(tx);
  if (rpc.Api.isSimulationError(simResult)) {
    return [];
  }

  const success = simResult as rpc.Api.SimulateTransactionSuccessResponse;
  const returnVal = success.result?.retval;
  if (!returnVal) return [];

  return scValToNative(returnVal) as string[];
}
