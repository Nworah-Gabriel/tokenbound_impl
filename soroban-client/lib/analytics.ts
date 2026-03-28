"use client";

import { useEffect } from "react";

const STORAGE_KEY = "crowdpass.analytics.v1";

type PageViewMap = Record<string, number>;
type EventPurchaseMap = Record<string, number>;

interface AnalyticsState {
  pageViews: PageViewMap;
  walletConnections: number;
  ticketsPurchased: number;
  revenueXlm: number;
  eventPurchases: EventPurchaseMap;
}

const DEFAULT_STATE: AnalyticsState = {
  pageViews: {},
  walletConnections: 0,
  ticketsPurchased: 0,
  revenueXlm: 0,
  eventPurchases: {},
};

export interface AnalyticsSnapshot {
  pageViews: PageViewMap;
  walletConnections: number;
  ticketsPurchased: number;
  revenueXlm: number;
  totalEvents: number;
  organizerRevenueXlm: number;
  organizerConversionRate: number;
  pageViewSeries: Array<{ name: string; views: number }>;
  eventSeries: Array<{ name: string; sold: number }>;
}

function canUseStorage() {
  return typeof window !== "undefined";
}

function readState(): AnalyticsState {
  if (!canUseStorage()) {
    return DEFAULT_STATE;
  }

  const raw = window.localStorage.getItem(STORAGE_KEY);
  if (!raw) {
    return DEFAULT_STATE;
  }

  try {
    return {
      ...DEFAULT_STATE,
      ...JSON.parse(raw),
    };
  } catch {
    return DEFAULT_STATE;
  }
}

function writeState(state: AnalyticsState) {
  if (!canUseStorage()) {
    return;
  }

  window.localStorage.setItem(STORAGE_KEY, JSON.stringify(state));
}

function updateState(mutator: (state: AnalyticsState) => AnalyticsState) {
  const next = mutator(readState());
  writeState(next);
  return next;
}

export function trackPageView(page: string) {
  updateState((state) => ({
    ...state,
    pageViews: {
      ...state.pageViews,
      [page]: (state.pageViews[page] || 0) + 1,
    },
  }));
}

export function trackWalletConnection() {
  updateState((state) => ({
    ...state,
    walletConnections: state.walletConnections + 1,
  }));
}

export function trackTicketPurchase(eventKey: string, quantity: number, revenueXlm: number) {
  updateState((state) => ({
    ...state,
    ticketsPurchased: state.ticketsPurchased + quantity,
    revenueXlm: Number((state.revenueXlm + revenueXlm).toFixed(2)),
    eventPurchases: {
      ...state.eventPurchases,
      [eventKey]: (state.eventPurchases[eventKey] || 0) + quantity,
    },
  }));
}

export function getAnalyticsSnapshot(): AnalyticsSnapshot {
  const state = readState();

  const pageViewSeries = Object.entries(state.pageViews).map(([name, views]) => ({
    name,
    views,
  }));

  const eventSeries = Object.entries(state.eventPurchases).map(([name, sold]) => ({
    name,
    sold,
  }));

  const totalViews = Object.values(state.pageViews).reduce(
    (sum, views) => sum + views,
    0
  );

  return {
    pageViews: state.pageViews,
    walletConnections: state.walletConnections,
    ticketsPurchased: state.ticketsPurchased,
    revenueXlm: state.revenueXlm,
    totalEvents: Math.max(eventSeries.length, 3),
    organizerRevenueXlm: state.revenueXlm,
    organizerConversionRate:
      totalViews === 0 ? 0 : Number(((state.ticketsPurchased / totalViews) * 100).toFixed(1)),
    pageViewSeries,
    eventSeries,
  };
}

export function useTrackPageView(page: string) {
  useEffect(() => {
    trackPageView(page);
  }, [page]);
}
