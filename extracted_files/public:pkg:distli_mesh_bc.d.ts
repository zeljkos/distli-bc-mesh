/* tslint:disable */
/* eslint-disable */
export function main(): void;
export class Blockchain {
  free(): void;
  constructor();
  add_message_and_mine(message: string, sender: string): string;
  mine_block_and_get(): string;
  add_validator(address: string, stake: number): void;
  add_transaction(from: string, to: string, amount: number): string;
  add_message(message: string, sender: string): string;
  deploy_trading_contract(owner: string): boolean;
  call_contract_buy(asset: string, quantity: number, price: number, sender: string): string;
  call_contract_sell(asset: string, quantity: number, price: number, sender: string): string;
  get_contract_order_book(): string;
  get_contract_trades(): string;
  mine_block(): boolean;
  get_chain_length(): number;
  get_pending_count(): number;
  get_validator_count(): number;
  get_latest_block_json(): string;
  get_transactions_json(): string;
  get_blockchain_summary(): string;
  add_p2p_transaction(tx_json: string): boolean;
  add_p2p_block(block_json: string): boolean;
  get_sync_summary(): string;
  prepare_enterprise_sync(): string;
  set_last_sync_block(_height: number): void;
}
export class OrderBook {
  free(): void;
  constructor();
  place_buy_order(trader: string, asset: string, quantity: number, price: number): string;
  place_sell_order(trader: string, asset: string, quantity: number, price: number): string;
  cancel_order(order_id: string): boolean;
  update_order_quantity(order_id: string, new_quantity: number): boolean;
  execute_cross_network_trade(asset: string, quantity: number, price: number, buyer: string, seller: string): string;
  clear_trader_orders(trader: string): number;
  get_order(order_id: string): string;
  get_order_book_json(): string;
  get_recent_trades_json(): string;
}

export type InitInput = RequestInfo | URL | Response | BufferSource | WebAssembly.Module;

export interface InitOutput {
  readonly memory: WebAssembly.Memory;
  readonly __wbg_blockchain_free: (a: number, b: number) => void;
  readonly blockchain_new: () => number;
  readonly blockchain_add_message_and_mine: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly blockchain_mine_block_and_get: (a: number) => [number, number];
  readonly blockchain_add_validator: (a: number, b: number, c: number, d: number) => void;
  readonly blockchain_add_transaction: (a: number, b: number, c: number, d: number, e: number, f: number) => [number, number];
  readonly blockchain_add_message: (a: number, b: number, c: number, d: number, e: number) => [number, number];
  readonly blockchain_deploy_trading_contract: (a: number, b: number, c: number) => number;
  readonly blockchain_call_contract_buy: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly blockchain_call_contract_sell: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly blockchain_get_contract_order_book: (a: number) => [number, number];
  readonly blockchain_get_contract_trades: (a: number) => [number, number];
  readonly blockchain_mine_block: (a: number) => number;
  readonly blockchain_get_chain_length: (a: number) => number;
  readonly blockchain_get_pending_count: (a: number) => number;
  readonly blockchain_get_validator_count: (a: number) => number;
  readonly blockchain_get_latest_block_json: (a: number) => [number, number];
  readonly blockchain_get_transactions_json: (a: number) => [number, number];
  readonly blockchain_get_blockchain_summary: (a: number) => [number, number];
  readonly blockchain_add_p2p_transaction: (a: number, b: number, c: number) => number;
  readonly blockchain_add_p2p_block: (a: number, b: number, c: number) => number;
  readonly blockchain_get_sync_summary: (a: number) => [number, number];
  readonly blockchain_prepare_enterprise_sync: (a: number) => [number, number];
  readonly blockchain_set_last_sync_block: (a: number, b: number) => void;
  readonly __wbg_orderbook_free: (a: number, b: number) => void;
  readonly orderbook_new: () => number;
  readonly orderbook_place_buy_order: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly orderbook_place_sell_order: (a: number, b: number, c: number, d: number, e: number, f: number, g: number) => [number, number];
  readonly orderbook_cancel_order: (a: number, b: number, c: number) => number;
  readonly orderbook_update_order_quantity: (a: number, b: number, c: number, d: number) => number;
  readonly orderbook_execute_cross_network_trade: (a: number, b: number, c: number, d: number, e: number, f: number, g: number, h: number, i: number) => [number, number];
  readonly orderbook_clear_trader_orders: (a: number, b: number, c: number) => number;
  readonly orderbook_get_order: (a: number, b: number, c: number) => [number, number];
  readonly orderbook_get_order_book_json: (a: number) => [number, number];
  readonly orderbook_get_recent_trades_json: (a: number) => [number, number];
  readonly main: () => void;
  readonly __wbindgen_free: (a: number, b: number, c: number) => void;
  readonly __wbindgen_malloc: (a: number, b: number) => number;
  readonly __wbindgen_realloc: (a: number, b: number, c: number, d: number) => number;
  readonly __wbindgen_export_3: WebAssembly.Table;
  readonly __wbindgen_start: () => void;
}

export type SyncInitInput = BufferSource | WebAssembly.Module;
/**
* Instantiates the given `module`, which can either be bytes or
* a precompiled `WebAssembly.Module`.
*
* @param {{ module: SyncInitInput }} module - Passing `SyncInitInput` directly is deprecated.
*
* @returns {InitOutput}
*/
export function initSync(module: { module: SyncInitInput } | SyncInitInput): InitOutput;

/**
* If `module_or_path` is {RequestInfo} or {URL}, makes a request and
* for everything else, calls `WebAssembly.instantiate` directly.
*
* @param {{ module_or_path: InitInput | Promise<InitInput> }} module_or_path - Passing `InitInput` directly is deprecated.
*
* @returns {Promise<InitOutput>}
*/
export default function __wbg_init (module_or_path?: { module_or_path: InitInput | Promise<InitInput> } | InitInput | Promise<InitInput>): Promise<InitOutput>;
