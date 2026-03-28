"use client";

import { useState } from "react";

import { useWallet } from "@/contexts/WalletContext";
import { buyTickets, isEventManagerConfigured } from "@/lib/soroban";
import { trackTicketPurchase } from "@/lib/analytics";

type CatalogEvent = {
  id: number;
  title: string;
  venue: string;
  date: string;
  priceXlm: number;
  remaining: number;
  description: string;
};

const EVENTS: CatalogEvent[] = [
  {
    id: 1,
    title: "Soroban Summit",
    venue: "Lagos Marina Hall",
    date: "Apr 18, 2026",
    priceXlm: 45,
    remaining: 84,
    description: "Protocol talks, live wallet demos, and on-chain ticket redemption.",
  },
  {
    id: 2,
    title: "Builder House",
    venue: "Yaba Creative Loft",
    date: "May 02, 2026",
    priceXlm: 28,
    remaining: 42,
    description: "Hands-on product sprints for event organizers and marketplace teams.",
  },
  {
    id: 3,
    title: "Campus Launch Night",
    venue: "UNILAG Open Theatre",
    date: "May 21, 2026",
    priceXlm: 18,
    remaining: 130,
    description: "Community launch event with badge drops and token-bound perks.",
  },
];

function totalPrice(priceXlm: number, quantity: number) {
  const subtotal = priceXlm * quantity;
  if (quantity >= 10) {
    return subtotal * 0.9;
  }
  if (quantity >= 5) {
    return subtotal * 0.95;
  }
  return subtotal;
}

export default function EventCatalog() {
  const { address, isConnected, isInstalled, connect } = useWallet();
  const [quantities, setQuantities] = useState<Record<number, number>>(
    EVENTS.reduce((acc, event) => ({ ...acc, [event.id]: 1 }), {})
  );
  const [status, setStatus] = useState<string>("");
  const [pendingEventId, setPendingEventId] = useState<number | null>(null);

  const updateQuantity = (eventId: number, value: number) => {
    setQuantities((current) => ({
      ...current,
      [eventId]: Math.max(1, value),
    }));
  };

  const handlePurchase = async (event: CatalogEvent) => {
    const quantity = quantities[event.id] || 1;

    if (!isConnected) {
      if (!isInstalled) {
        setStatus("Install Freighter to continue with ticket purchases.");
        return;
      }

      await connect();
    }

    setPendingEventId(event.id);
    setStatus("");

    try {
      if (address && isEventManagerConfigured()) {
        await buyTickets({
          buyer: address,
          eventId: event.id,
          quantity: BigInt(quantity),
        });
      }

      const revenue = totalPrice(event.priceXlm, quantity);
      trackTicketPurchase(event.title, quantity, revenue);
      setStatus(
        `Reserved ${quantity} ticket${quantity > 1 ? "s" : ""} for ${event.title}.`
      );
    } catch (error) {
      const message =
        error instanceof Error ? error.message : "Ticket purchase failed.";
      setStatus(message);
    } finally {
      setPendingEventId(null);
    }
  };

  return (
    <section className="space-y-6">
      <div className="grid gap-5 lg:grid-cols-3">
        {EVENTS.map((event) => {
          const quantity = quantities[event.id] || 1;
          const price = totalPrice(event.priceXlm, quantity);

          return (
            <article
              key={event.id}
              className="flex h-full flex-col rounded-[28px] border border-white/10 bg-white/5 p-6 shadow-xl shadow-black/20"
            >
              <div className="flex items-start justify-between gap-4">
                <div>
                  <p className="text-sm uppercase tracking-[0.3em] text-orange-200/70">
                    Event #{event.id}
                  </p>
                  <h2 className="mt-2 text-2xl font-semibold text-white">
                    {event.title}
                  </h2>
                </div>
                <span className="rounded-full bg-emerald-500/10 px-3 py-1 text-sm text-emerald-200">
                  {event.remaining} left
                </span>
              </div>

              <p className="mt-4 text-sm leading-6 text-zinc-300">{event.description}</p>

              <dl className="mt-6 space-y-3 text-sm text-zinc-300">
                <div className="flex items-center justify-between">
                  <dt>Date</dt>
                  <dd>{event.date}</dd>
                </div>
                <div className="flex items-center justify-between">
                  <dt>Venue</dt>
                  <dd>{event.venue}</dd>
                </div>
                <div className="flex items-center justify-between">
                  <dt>Base price</dt>
                  <dd>{event.priceXlm} XLM</dd>
                </div>
              </dl>

              <div className="mt-6 rounded-2xl border border-white/10 bg-black/20 p-4">
                <label
                  htmlFor={`quantity-${event.id}`}
                  className="text-sm font-medium text-zinc-200"
                >
                  Quantity
                </label>
                <div className="mt-3 flex items-center gap-3">
                  <button
                    type="button"
                    onClick={() => updateQuantity(event.id, quantity - 1)}
                    className="flex h-11 w-11 items-center justify-center rounded-2xl border border-white/15 bg-white/5 text-xl text-white transition hover:bg-white/10"
                    aria-label={`Decrease quantity for ${event.title}`}
                  >
                    -
                  </button>
                  <input
                    id={`quantity-${event.id}`}
                    type="number"
                    min={1}
                    max={event.remaining}
                    value={quantity}
                    onChange={(eventInput) =>
                      updateQuantity(event.id, Number(eventInput.target.value))
                    }
                    className="h-11 w-20 rounded-2xl border border-white/10 bg-zinc-950 px-4 text-center text-white"
                  />
                  <button
                    type="button"
                    onClick={() => updateQuantity(event.id, quantity + 1)}
                    className="flex h-11 w-11 items-center justify-center rounded-2xl border border-white/15 bg-white/5 text-xl text-white transition hover:bg-white/10"
                    aria-label={`Increase quantity for ${event.title}`}
                  >
                    +
                  </button>
                </div>

                <div className="mt-4 flex items-center justify-between text-sm text-zinc-300">
                  <span>Batch price</span>
                  <span>{price.toFixed(2)} XLM</span>
                </div>
                {quantity >= 5 && (
                  <p className="mt-2 text-xs uppercase tracking-[0.24em] text-orange-200/80">
                    {quantity >= 10 ? "10%" : "5%"} group discount applied
                  </p>
                )}
              </div>

              <button
                type="button"
                onClick={() => handlePurchase(event)}
                disabled={pendingEventId === event.id}
                className="mt-6 w-full rounded-2xl bg-[#FF5722] px-5 py-4 text-base font-semibold text-white transition hover:bg-[#F4511E] disabled:cursor-not-allowed disabled:opacity-60"
              >
                {pendingEventId === event.id ? "Processing..." : "Purchase tickets"}
              </button>
            </article>
          );
        })}
      </div>

      {status && (
        <div className="rounded-2xl border border-white/10 bg-white/5 px-4 py-3 text-sm text-zinc-200">
          {status}
        </div>
      )}
    </section>
  );
}
