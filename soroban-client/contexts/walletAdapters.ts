import {
  isConnected as freighterIsConnected,
  requestAccess as freighterRequestAccess,
  signTransaction as freighterSignTransaction,
} from "@stellar/freighter-api";

export type WalletProviderId = "freighter" | "xbull" | "albedo" | "walletconnect";

export interface WalletProviderInfo {
  id: WalletProviderId;
  name: string;
  installUrl: string;
  installed: boolean;
  description: string;
}

declare global {
  interface Window {
    freighter?: unknown;
    xbull?: {
      getPublicKey?: () => Promise<string>;
      publicKey?: () => Promise<string>;
      connect?: () => Promise<{ publicKey: string }>;
      signTransaction?: (xdr: string, opts: Record<string, unknown>) => Promise<{ signedXdr: string }>;
      request?: (payload: unknown) => Promise<unknown>;
    };
    Albedo?: {
      publicKey?: () => Promise<string>;
      signTransaction?: (params: { xdr: string; network_passphrase: string }) => Promise<{ signed_xdr: string }>;
    };
    albedo?: {
      publicKey?: () => Promise<string>;
      signTransaction?: (params: { xdr: string; network_passphrase: string }) => Promise<{ signed_xdr: string }>;
    };
    WalletConnectProvider?: unknown;
  }
}

export const ALL_WALLET_PROVIDERS: Array<WalletProviderInfo> = [
  {
    id: "freighter",
    name: "Freighter",
    installUrl: "https://www.freighter.app/",
    installed: false,
    description: "Browser extension for Stellar accounts.",
  },
  {
    id: "xbull",
    name: "xBull",
    installUrl: "https://wallet.xbull.app/",
    installed: false,
    description: "Stellar wallet extension and web wallet.",
  },
  {
    id: "albedo",
    name: "Albedo",
    installUrl: "https://albedo.link/",
    installed: false,
    description: "Decentralized Stellar wallet interface.",
  },
  {
    id: "walletconnect",
    name: "WalletConnect",
    installUrl: "https://walletconnect.com/",
    installed: true,
    description: "Connect using WalletConnect QR code session.",
  },
];

export async function detectAvailableWalletProviders(): Promise<WalletProviderInfo[]> {
  if (typeof window === "undefined") {
    return ALL_WALLET_PROVIDERS.map((wallet) => ({ ...wallet, installed: false }));
  }

  const freighterInstalled = Boolean(window.freighter);
  const xbullInstalled = Boolean(window.xbull);
  const albedoInstalled = Boolean(window.Albedo || window.albedo);

  let freighterActive = false;
  if (freighterInstalled) {
    try {
      const result = await freighterIsConnected();
      freighterActive = !!result?.isConnected;
    } catch {
      freighterActive = true; // still provide path if extension object exists
    }
  }

  return ALL_WALLET_PROVIDERS.map((wallet) => {
    switch (wallet.id) {
      case "freighter":
        return { ...wallet, installed: freighterInstalled || freighterActive };
      case "xbull":
        return { ...wallet, installed: xbullInstalled };
      case "albedo":
        return { ...wallet, installed: albedoInstalled };
      case "walletconnect":
        return { ...wallet, installed: true };
      default:
        return { ...wallet, installed: false };
    }
  });
}

export async function connectWallet(providerId: WalletProviderId): Promise<string> {
  if (typeof window === "undefined") {
    throw new Error("Wallet connection unavailable on server side.");
  }

  switch (providerId) {
    case "freighter": {
      if (!window.freighter) {
        throw new Error("Freighter is not installed.");
      }
      const response = await freighterRequestAccess();
      if (response?.address) {
        return response.address;
      }
      throw new Error(response?.error || "Failed to connect to Freighter.");
    }

    case "xbull": {
      if (!window.xbull) {
        throw new Error("xBull wallet is not installed.");
      }
      const publicKeyFn = window.xbull.getPublicKey || window.xbull.publicKey;
      if (publicKeyFn) {
        return await publicKeyFn();
      }
      if (window.xbull.connect) {
        const result = await window.xbull.connect();
        if (result?.publicKey) {
          return result.publicKey;
        }
      }
      throw new Error("xBull wallet could not provide public key.");
    }

    case "albedo": {
      const albedo = window.Albedo || window.albedo;
      if (!albedo || !albedo.publicKey) {
        throw new Error("Albedo wallet is not installed or unavailable.");
      }
      const pk = await albedo.publicKey();
      if (!pk) throw new Error("Albedo did not return a public key.");
      return pk;
    }

    case "walletconnect": {
      // WalletConnect is supported via external wallet. We expose a hint to open user flow.
      throw new Error("WalletConnect sign-in flow is not implemented in this build. Use a wallet provider extension.");
    }

    default:
      throw new Error("Unsupported wallet provider.");
  }
}

export async function signTransactionWithProvider(
  providerId: WalletProviderId,
  txXdr: string,
  options: { networkPassphrase: string; address: string }
): Promise<string> {
  if (typeof window === "undefined") {
    throw new Error("Cannot sign transaction on server side.");
  }

  switch (providerId) {
    case "freighter": {
      const result = await freighterSignTransaction(txXdr, {
        networkPassphrase: options.networkPassphrase,
        address: options.address,
      });
      if (!result?.signedTxXdr) {
        throw new Error("Freighter failed to sign transaction.");
      }
      return result.signedTxXdr;
    }

    case "xbull": {
      if (!window.xbull?.signTransaction) {
        throw new Error("xBull does not support signTransaction from this interface.");
      }
      const res = await window.xbull.signTransaction(txXdr, {
        networkPassphrase: options.networkPassphrase,
        address: options.address,
      });
      if (!res?.signedXdr) {
        throw new Error("xBull failed to sign transaction.");
      }
      return res.signedXdr;
    }

    case "albedo": {
      const albedo = window.Albedo || window.albedo;
      if (!albedo?.signTransaction) {
        throw new Error("Albedo does not support contract transaction signing through this interface.");
      }
      const payload = await albedo.signTransaction({
        xdr: txXdr,
        network_passphrase: options.networkPassphrase,
      });
      if (!payload?.signed_xdr) {
        throw new Error("Albedo failed to sign transaction.");
      }
      return payload.signed_xdr;
    }

    case "walletconnect": {
      // WalletConnect can be integrated with a provider in a separate module.
      throw new Error("WalletConnect transaction signing is currently not implemented in this repository.");
    }

    default:
      throw new Error("Unsupported wallet provider.");
  }
}
