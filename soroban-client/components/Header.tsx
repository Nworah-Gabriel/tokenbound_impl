"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";
import { Menu, X } from "lucide-react";
import { useTranslations, useLocale } from "next-intl";

import { useWallet } from "@/contexts/WalletContext";
import type { WalletProviderId } from "@/contexts/walletAdapters";
import LanguageSwitcher from "@/components/LanguageSwitcher";

export default function Header() {
  const t = useTranslations("nav");
  const tWallet = useTranslations("wallet");
  const locale = useLocale();

  const {
    address,
    providerId,
    providerName,
    availableProviders,
    isConnected,
    isInstalled,
    connect,
    disconnect,
    setProviderId,
  } = useWallet();

  const pathname = usePathname();
  const [isMenuOpen, setIsMenuOpen] = useState(false);
  const [isWalletModalOpen, setIsWalletModalOpen] = useState(false);

  const NAV_LINKS = [
    { href: `/${locale}`, label: t("home") },
    { href: `/${locale}/events`, label: t("events") },
    { href: `/${locale}/analytics`, label: t("marketplace") },
  ];

  const openWalletModal = () => setIsWalletModalOpen(true);
  const closeWalletModal = () => setIsWalletModalOpen(false);
  const closeMenu = () => setIsMenuOpen(false);

  const handleProviderSelect = async (selectedProviderId: WalletProviderId) => {
    setProviderId(selectedProviderId);
    try {
      await connect(selectedProviderId);
    } catch {
      /* connect() already surfaced error */
    } finally {
      closeWalletModal();
    }
  };

  const formatAddress = (value: string) =>
    `${value.substring(0, 4)}...${value.substring(value.length - 4)}`;

  const handleConnect = async () => {
    if (!isInstalled) {
      openWalletModal();
      return;
    }
    try {
      await connect(providerId);
    } catch {
      openWalletModal();
    }
  };

  const handleMenuAction = async (action?: () => Promise<void> | void) => {
    if (action) {
      await action();
    }
    closeMenu();
  };

  useEffect(() => {
    document.body.style.overflow = isMenuOpen ? "hidden" : "";
    return () => {
      document.body.style.overflow = "";
    };
  }, [isMenuOpen]);

  useEffect(() => {
    closeMenu();
  }, [pathname]);

  return (
    <header className="absolute left-0 right-0 top-0 z-100 flex justify-center px-4 pt-8">
      <div className="flex w-full max-w-6xl items-center justify-between rounded-2xl bg-[#525252] px-6 py-4 shadow-lg backdrop-blur-sm">
        <Link href={`/${locale}`} className="flex items-center gap-2 text-white">
          <div className="flex items-center gap-2 text-2xl font-bold">
            <svg width="32" height="32" viewBox="0 0 32 32" fill="none" xmlns="http://www.w3.org/2000/svg">
              <path d="M4 10H14V14H8V22H14V26H4V10Z" fill="white" />
              <path d="M18 10H28V14H22V26H18V10Z" fill="white" />
            </svg>
            CrowdPass
          </div>
        </Link>

        {/* Desktop Navigation */}
        <nav className="hidden items-center gap-8 md:flex">
          {NAV_LINKS.map((link) => (
            <Link
              key={link.href}
              href={link.href}
              className={`font-medium transition ${
                pathname === link.href ? "text-white" : "text-gray-200 hover:text-white"
              }`}
            >
              {link.label}
            </Link>
          ))}
          {isConnected && (
            <Link
              href={`/${locale}/dashboard`}
              className="font-medium text-gray-200 transition hover:text-white"
            >
              {t("dashboard")}
            </Link>
          )}
        </nav>

        <div className="hidden items-center gap-4 md:flex">
          <LanguageSwitcher />
          {isConnected ? (
            <div className="flex items-center gap-4">
              <span className="rounded-md bg-white/10 px-3 py-1 font-mono text-sm text-gray-300">
                {formatAddress(address!)}
              </span>
              <button
                type="button"
                onClick={disconnect}
                className="rounded-lg border border-gray-400 px-6 py-2 font-medium text-white transition hover:bg-white/10"
              >
                {t("disconnect")}
              </button>
            </div>
          ) : (
            <button
              type="button"
              onClick={handleConnect}
              className="rounded-lg border border-gray-400 px-6 py-2 font-medium text-white transition hover:bg-white/10"
            >
              {isInstalled ? t("connect", { provider: providerName }) : t("selectWallet")}
            </button>
          )}
          <Link
            href={`/${locale}/create-event`}
            className="inline-block rounded-lg bg-[#FF5722] px-6 py-2 text-center font-bold text-white shadow-md transition hover:bg-[#F4511E]"
          >
            {t("createEvent")}
          </Link>
        </div>

        <button
          type="button"
          onClick={() => setIsMenuOpen((current) => !current)}
          className="flex items-center justify-center rounded-lg p-2 transition hover:bg-white/10 md:hidden"
          aria-label={t("toggleMenu")}
          aria-expanded={isMenuOpen}
          aria-controls="mobile-menu"
        >
          {isMenuOpen ? <X size={24} className="text-white" /> : <Menu size={24} className="text-white" />}
        </button>
      </div>

      {/* Mobile overlay */}
      <div
        className={`fixed inset-0 z-40 bg-black/50 transition-opacity duration-300 md:hidden ${
          isMenuOpen ? "pointer-events-auto opacity-100" : "pointer-events-none opacity-0"
        }`}
        onClick={closeMenu}
        role="presentation"
      />

      {/* Mobile menu */}
      <nav
        id="mobile-menu"
        role="dialog"
        aria-label={t("menu")}
        className={`fixed left-0 right-0 top-0 z-50 w-full max-w-full origin-top bg-[#525252] shadow-lg transition-all duration-300 md:hidden ${
          isMenuOpen
            ? "pointer-events-auto translate-y-0 opacity-100"
            : "pointer-events-none -translate-y-full opacity-0"
        }`}
        style={{ paddingTop: "env(safe-area-inset-top)" }}
      >
        <div className="flex items-center justify-between px-4 py-4">
          <div className="text-xl font-bold text-white">{t("menu")}</div>
          <button
            type="button"
            onClick={closeMenu}
            className="rounded-lg p-2 transition hover:bg-white/10"
            aria-label={t("closeMenu")}
          >
            <X size={24} className="text-white" />
          </button>
        </div>

        <div className="border-t border-gray-600" />

        <div className="space-y-4 px-4 py-6">
          <div className="space-y-3">
            {NAV_LINKS.map((link) => (
              <Link
                key={link.href}
                href={link.href}
                className={`block rounded-2xl px-4 py-3 text-lg font-medium transition ${
                  pathname === link.href
                    ? "bg-white/10 text-white"
                    : "text-gray-200 hover:bg-white/5 hover:text-white"
                }`}
                onClick={() => handleMenuAction()}
              >
                {link.label}
              </Link>
            ))}
            {isConnected && (
              <Link
                href={`/${locale}/dashboard`}
                className="block rounded-2xl px-4 py-3 text-lg font-medium text-gray-200 transition hover:bg-white/5 hover:text-white"
                onClick={() => handleMenuAction()}
              >
                {t("dashboard")}
              </Link>
            )}
          </div>

          <div className="border-t border-gray-600 py-4" />

          <div className="space-y-3">
            {isConnected ? (
              <>
                <div className="rounded-2xl bg-white/10 px-4 py-3 font-mono text-sm text-gray-300">
                  {formatAddress(address!)}
                </div>
                <button
                  type="button"
                  onClick={() => handleMenuAction(disconnect)}
                  className="w-full rounded-2xl border border-gray-400 px-4 py-4 font-medium text-white transition hover:bg-white/10"
                >
                  {t("disconnect")}
                </button>
              </>
            ) : (
              <button
                type="button"
                onClick={() => handleMenuAction(handleConnect)}
                className="w-full rounded-2xl border border-gray-400 px-4 py-4 font-medium text-white transition hover:bg-white/10"
              >
                {isInstalled ? t("connect", { provider: providerName }) : t("selectWallet")}
              </button>
            )}
            <Link
              href={`/${locale}/create-event`}
              onClick={() => handleMenuAction()}
              className="block w-full rounded-2xl bg-[#FF5722] px-4 py-4 text-center font-bold text-white shadow-md transition hover:bg-[#F4511E]"
            >
              {t("createEvent")}
            </Link>
            <div className="flex justify-center pt-2">
              <LanguageSwitcher />
            </div>
          </div>
        </div>
      </nav>

      {/* Wallet Selection Modal */}
      {isWalletModalOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 p-4">
          <div className="w-full max-w-md rounded-xl bg-[#252525] p-4 text-white shadow-xl">
            <div className="mb-3 flex items-center justify-between">
              <h3 className="text-lg font-bold">{tWallet("chooseProvider")}</h3>
              <button
                type="button"
                onClick={closeWalletModal}
                className="text-white hover:text-gray-300"
              >
                <X size={20} />
              </button>
            </div>
            <div className="space-y-2">
              {availableProviders.map((provider) => (
                <button
                  key={provider.id}
                  type="button"
                  onClick={() => {
                    if (provider.installed) {
                      void handleProviderSelect(provider.id);
                    } else {
                      window.open(provider.installUrl, "_blank");
                    }
                  }}
                  className={`w-full rounded-lg border p-3 text-left transition ${
                    provider.id === providerId ? "border-blue-400" : "border-gray-600"
                  } ${provider.installed ? "cursor-pointer hover:border-blue-300" : "cursor-pointer opacity-70"}`}
                >
                  <div className="flex items-center justify-between">
                    <div>
                      <div className="font-semibold">{provider.name}</div>
                      <div className="text-xs text-gray-300">{provider.description}</div>
                    </div>
                    <div className="text-xs">
                      {provider.installed ? tWallet("available") : tWallet("install")}
                    </div>
                  </div>
                </button>
              ))}
            </div>
            <div className="mt-4 text-sm text-gray-300">{tWallet("notInstalled")}</div>
          </div>
        </div>
      )}
    </header>
  );
}
