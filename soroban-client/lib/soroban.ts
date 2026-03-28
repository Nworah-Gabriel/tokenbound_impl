import { Horizon, TransactionBuilder, Operation, Networks } from "@stellar/stellar-sdk";
import { nativeToScVal } from "@stellar/stellar-base";
import { signTransaction } from "@stellar/freighter-api";
const { Server } = Horizon;
import StellarSdk from "@stellar/stellar-sdk";
import { nativeToScVal, scValToNative } from "@stellar/stellar-base";
import { signTransaction } from "@stellar/freighter-api";

// "@stellar/stellar-sdk" has a default export that bundles several
// constructors; we pull out the pieces we need for clarity.
const { Server, TransactionBuilder, Operation, Networks, SorobanRpc } = StellarSdk;

// Configuration helpers – prefer environment variables so they can be swapped
// for different networks (testnet / preview / mainnet) without changing code.
const HORIZON_URL =
  process.env.NEXT_PUBLIC_HORIZON_URL || "https://horizon-testnet.stellar.org";
const SOROBAN_RPC_URL =
  process.env.NEXT_PUBLIC_SOROBAN_RPC_URL || "https://soroban-testnet.stellar.org";
const NETWORK_PASSPHRASE =
  process.env.NEXT_PUBLIC_NETWORK_PASSPHRASE || Networks.TESTNET;
// contract ID of the deployed EventManager; set this in .env.local
const EVENT_MANAGER_CONTRACT =
  process.env.NEXT_PUBLIC_EVENT_MANAGER_CONTRACT || "<MISSING_CONTRACT_ID>";

export interface CreateEventParams {
  organizer: string; // freighter address
  theme: string;
  eventType: string;
  startTimeUnix: number;
  endTimeUnix: number;
  ticketPrice: bigint;
  totalTickets: bigint;
  paymentToken: string; // contract address for token used for payment
}

export interface BuyTicketsParams {
  buyer: string;
  eventId: number;
  quantity: bigint;
}

export function isEventManagerConfigured() {
  return EVENT_MANAGER_CONTRACT !== "<MISSING_CONTRACT_ID>";
}

/**
 * Builds, signs (via Freighter) and submits a transaction to create a new
 * event using the EventManager Soroban contract.
 *
 * The caller must have a Freighter wallet connected and the supplied
 * `organizer` address must match the source account in the wallet.
 */
export async function createEvent(params: CreateEventParams) {
  if (!isEventManagerConfigured()) {
    throw new Error(
      "EVENT_MANAGER_CONTRACT is not configured. Set NEXT_PUBLIC_EVENT_MANAGER_CONTRACT in your env."
    );
  }

  const server = new Server(HORIZON_URL);

  // load account to obtain current sequence number
  const sourceAccount = await server.loadAccount(params.organizer);

  // use the standard base fee
  const fee = await server.fetchBaseFee();

  // prepare soroban arguments converting native JS values to ScVals
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

  const operation = Operation.invokeContractFunction({
    contract: EVENT_MANAGER_CONTRACT,
    function: "create_event",
    args,
  });

  const tx = new TransactionBuilder(sourceAccount, {
    fee: fee.toString(),
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(operation)
    .setTimeout(30)
    .build();

  const txXdr = tx.toXDR();

  // ask Freighter to sign
  const { signedTxXdr } = await signTransaction(txXdr, {
    networkPassphrase: NETWORK_PASSPHRASE,
    address: params.organizer,
  });

  // submit to horizon and return the result
  const signedTx = TransactionBuilder.fromXDR(signedTxXdr, NETWORK_PASSPHRASE);
  return await server.submitTransaction(signedTx as any);
}

export async function buyTickets(params: BuyTicketsParams) {
  if (!isEventManagerConfigured()) {
    throw new Error(
      "EVENT_MANAGER_CONTRACT is not configured. Set NEXT_PUBLIC_EVENT_MANAGER_CONTRACT in your env."
    );
  }

  const server = new Server(HORIZON_URL);
  const sourceAccount = await server.loadAccount(params.buyer);
  const fee = await server.fetchBaseFee();

  const args = [
    nativeToScVal(params.buyer, { type: "address" }),
    nativeToScVal(params.eventId, { type: "u32" }),
    nativeToScVal(params.quantity, { type: "u128" }),
  ];

  const operation = Operation.invokeContractFunction({
    contract: EVENT_MANAGER_CONTRACT,
    function: "purchase_tickets",
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

async function simulateAndInvoke(
  caller: string,
  functionName: string,
  args: ReturnType<typeof nativeToScVal>[]
) {
  const rpc = new SorobanRpc.Server(SOROBAN_RPC_URL);
  const server = new Server(HORIZON_URL);
  const sourceAccount = await server.loadAccount(caller);
  const fee = await server.fetchBaseFee();

  const operation = Operation.invokeContractFunction({
    contract: EVENT_MANAGER_CONTRACT,
    function: functionName,
    args,
  });

  const tx = new TransactionBuilder(sourceAccount, {
    fee: fee.toString(),
    networkPassphrase: NETWORK_PASSPHRASE,
  })
    .addOperation(operation)
    .setTimeout(30)
    .build();

  const txXdr = tx.toXDR();
  const { signedTxXdr } = await signTransaction(txXdr, {
    networkPassphrase: NETWORK_PASSPHRASE,
    address: params.buyer,
  const simResult = await rpc.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation failed: ${simResult.error}`);
  }

  const preparedTx = SorobanRpc.assembleTransaction(tx, simResult).build();
  const { signedTxXdr } = await signTransaction(preparedTx.toXDR(), {
    networkPassphrase: NETWORK_PASSPHRASE,
    address: caller,
  });

  return await server.submitTransaction(signedTxXdr);
}

/** Read all events from the contract (view call, no signing needed). */
export async function getAllEvents(): Promise<Event[]> {
  const rpc = new SorobanRpc.Server(SOROBAN_RPC_URL);
  const server = new Server(HORIZON_URL);

  // Use a dummy fee account for simulation – we just need a read
  const operation = Operation.invokeContractFunction({
    contract: EVENT_MANAGER_CONTRACT,
    function: "get_all_events",
    args: [],
  });

  // Build a transaction with a placeholder account (won't be submitted)
  const tx = new TransactionBuilder(
    { accountId: () => EVENT_MANAGER_CONTRACT, sequenceNumber: () => "0", incrementSequenceNumber: () => {} } as any,
    { fee: "100", networkPassphrase: NETWORK_PASSPHRASE }
  )
    .addOperation(operation)
    .setTimeout(30)
    .build();

  const simResult = await rpc.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simResult)) {
    throw new Error(`Simulation failed: ${simResult.error}`);
  }

  const returnVal = (simResult as SorobanRpc.Api.SimulateTransactionSuccessResponse).result?.retval;
  if (!returnVal) return [];

  const raw = scValToNative(returnVal) as any[];
  return raw.map((e: any) => ({
    id: Number(e.id),
    theme: e.theme,
    organizer: e.organizer,
    event_type: e.event_type,
    total_tickets: BigInt(e.total_tickets),
    tickets_sold: BigInt(e.tickets_sold),
    ticket_price: BigInt(e.ticket_price),
    start_date: Number(e.start_date),
    end_date: Number(e.end_date),
    is_canceled: e.is_canceled,
    ticket_nft_addr: e.ticket_nft_addr,
    payment_token: e.payment_token,
  }));
}

/** Cancel an event. Caller must be the organizer. */
export async function cancelEvent(organizer: string, eventId: number) {
  return simulateAndInvoke(organizer, "cancel_event", [
    nativeToScVal(eventId, { type: "u32" }),
  ]);
}

/** Update event details. Caller must be the organizer. */
export async function updateEvent(params: UpdateEventParams) {
  const toOption = (val: any, type: string) =>
    val !== undefined
      ? nativeToScVal({ Some: nativeToScVal(val, { type }) }, { type: "option" })
      : nativeToScVal(null, { type: "option" });

  return simulateAndInvoke(params.organizer, "update_event", [
    nativeToScVal(params.event_id, { type: "u32" }),
    toOption(params.theme, "string"),
    toOption(params.ticket_price, "i128"),
    toOption(params.total_tickets, "u128"),
    toOption(params.start_date, "u64"),
    toOption(params.end_date, "u64"),
  ]);
}

/** Claim funds after event completion. Caller must be the organizer. */
export async function claimFunds(organizer: string, eventId: number) {
  return simulateAndInvoke(organizer, "withdraw_funds", [
    nativeToScVal(eventId, { type: "u32" }),
  ]);
}

/** Get attendees (buyers) for an event. */
export async function getEventAttendees(eventId: number): Promise<string[]> {
  const rpc = new SorobanRpc.Server(SOROBAN_RPC_URL);

  const operation = Operation.invokeContractFunction({
    contract: EVENT_MANAGER_CONTRACT,
    function: "get_event_buyers",
    args: [nativeToScVal(eventId, { type: "u32" })],
  });

  const tx = new TransactionBuilder(
    { accountId: () => EVENT_MANAGER_CONTRACT, sequenceNumber: () => "0", incrementSequenceNumber: () => {} } as any,
    { fee: "100", networkPassphrase: NETWORK_PASSPHRASE }
  )
    .addOperation(operation)
    .setTimeout(30)
    .build();

  const simResult = await rpc.simulateTransaction(tx);
  if (SorobanRpc.Api.isSimulationError(simResult)) return [];

  const returnVal = (simResult as SorobanRpc.Api.SimulateTransactionSuccessResponse).result?.retval;
  if (!returnVal) return [];

  return scValToNative(returnVal) as string[];
}
