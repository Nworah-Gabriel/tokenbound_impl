"use client";

import React, { useState } from "react";
import { useRouter } from "next/navigation";
import { useWallet } from "@/contexts/WalletContext";
import { createEvent } from "@/lib/soroban";
import { useForm } from "react-hook-form";
import { zodResolver } from "@hookform/resolvers/zod";
import * as z from "zod";
import Header from "@/components/Header";

const eventSchema = z.object({
  theme: z.string().min(1, "Event name required"),
  description: z.string().optional(),
  startDate: z.string().min(1, "Start date is required"),
  endDate: z.string().min(1, "End date is required"),
  price: z.coerce.number({ invalid_type_error: "Price must be a number" }).min(0, "Price cannot be negative").max(1000000, "Price is too high"),
  tickets: z.coerce.number({ invalid_type_error: "Tickets must be a number" }).int("Must be a positive integer").positive("Must be a positive integer"),
}).superRefine((data, ctx) => {
  if (data.startDate) {
    const start = new Date(data.startDate).getTime();
    if (start <= Date.now()) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ["startDate"],
        message: "Start date must be in the future",
      });
    }
  }

  if (data.startDate && data.endDate) {
    const start = new Date(data.startDate).getTime();
    const end = new Date(data.endDate).getTime();
    if (end <= start) {
      ctx.addIssue({
        code: z.ZodIssueCode.custom,
        path: ["endDate"],
        message: "End date must be after start date",
      });
    }
  }
});

type EventFormData = z.infer<typeof eventSchema>;

export default function CreateEventPage() {
  const router = useRouter();
  const { address, isConnected, isInstalled, connect } = useWallet();

  const [image, setImage] = useState<File | null>(null);

  const {
    register,
    handleSubmit,
    formState: { errors, isValid, isSubmitting },
  } = useForm<EventFormData>({
    resolver: zodResolver(eventSchema),
    mode: "onChange",
  });

  const [successMsg, setSuccessMsg] = useState("");
  const [errorMsg, setErrorMsg] = useState("");

  const onSubmit = async (data: EventFormData) => {
    if (!address) {
      if (isInstalled) {
        await connect();
      } else {
        alert("Please install Freighter to create an event.");
        return;
      }
    }

    setErrorMsg("");
    setSuccessMsg("");

    try {
      const organizer = address!;
      const startUnix = Math.floor(new Date(data.startDate).getTime() / 1000);
      const endUnix = Math.floor(new Date(data.endDate).getTime() / 1000);
      const ticketPrice = BigInt(Math.floor(data.price * 1_000_000));
      const totalTickets = BigInt(data.tickets);

      // for simplicity we use zero address as payment token; replace with real
      // token contract address or allow user selection later.
      const paymentToken = "0000000000000000000000000000000000000000000000000000000000000000";

      const res = await createEvent({
        organizer,
        theme: data.theme,
        eventType: data.description || "",
        startTimeUnix: startUnix,
        endTimeUnix: endUnix,
        ticketPrice,
        totalTickets,
        paymentToken,
      });

      console.log("transaction result", res);
      setSuccessMsg("Event created (tx " + res.hash + ")");
      // Optionally redirect to dashboard or home after creation
      setTimeout(() => router.push("/"), 3000);
    } catch (err: any) {
      console.error(err);
      setErrorMsg(err.message || "unknown error");
    }
  };

  return (
    <div className="bg-[#18181B] min-h-screen text-white font-sans selection:bg-[#FF5722] selection:text-white flex flex-col">
      <Header />
      <main className="grow flex flex-col items-center justify-center pt-32 pb-20 px-4">
        <div className="w-full max-w-2xl bg-white/5 backdrop-blur-md border border-white/10 rounded-2xl p-8 shadow-2xl">
          <div className="mb-8 text-center">
            <h1 className="text-3xl md:text-4xl font-bold bg-clip-text text-transparent bg-gradient-to-r from-white to-gray-400">
              Create New Event
            </h1>
            <p className="text-gray-400 mt-2">
              Launch your decentralized event with secure ticketing.
            </p>
          </div>

          {successMsg && (
            <div className="bg-green-500/10 border border-green-500/20 text-green-400 p-4 rounded-xl mb-6 flex items-center gap-3">
              <svg className="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M5 13l4 4L19 7" /></svg>
              {successMsg}
            </div>
          )}
          {errorMsg && (
            <div className="bg-red-500/10 border border-red-500/20 text-red-400 p-4 rounded-xl mb-6 flex items-center gap-3">
              <svg className="w-5 h-5 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24"><path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M12 8v4m0 4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" /></svg>
              {errorMsg}
            </div>
          )}

          <form onSubmit={handleSubmit(onSubmit)} className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="md:col-span-2">
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Event Name
                </label>
                <input
                  type="text"
                  placeholder="e.g., Stellar DevCon 2026"
                  {...register("theme")}
                  className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-[#FF5722]/50 focus:border-[#FF5722] transition"
                />
                {errors.theme && (
                  <p className="text-red-400 text-sm mt-1.5 flex items-center gap-1"><span className="text-xs">⚠️</span> {errors.theme.message}</p>
                )}
              </div>

              <div className="md:col-span-2">
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Description
                </label>
                <textarea
                  {...register("description")}
                  placeholder="Tell your audience about the event..."
                  className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-[#FF5722]/50 focus:border-[#FF5722] transition"
                  rows={4}
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Start Date &amp; Time
                </label>
                <input
                  type="datetime-local"
                  {...register("startDate")}
                  className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-[#FF5722]/50 focus:border-[#FF5722] transition [color-scheme:dark]"
                />
                {errors.startDate && (
                  <p className="text-red-400 text-sm mt-1.5 flex items-center gap-1"><span className="text-xs">⚠️</span> {errors.startDate.message}</p>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  End Date &amp; Time
                </label>
                <input
                  type="datetime-local"
                  {...register("endDate")}
                  className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-[#FF5722]/50 focus:border-[#FF5722] transition [color-scheme:dark]"
                />
                {errors.endDate && (
                  <p className="text-red-400 text-sm mt-1.5 flex items-center gap-1"><span className="text-xs">⚠️</span> {errors.endDate.message}</p>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Ticket Price (XLM)
                </label>
                <div className="relative">
                    <span className="absolute left-4 top-1/2 -translate-y-1/2 text-gray-400 font-medium">XLM</span>
                    <input
                    type="number"
                    step="0.000001"
                    placeholder="0.00"
                    {...register("price")}
                    className="w-full bg-black/20 border border-white/10 rounded-xl pl-12 pr-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-[#FF5722]/50 focus:border-[#FF5722] transition"
                    />
                </div>
                {errors.price && (
                  <p className="text-red-400 text-sm mt-1.5 flex items-center gap-1"><span className="text-xs">⚠️</span> {errors.price.message}</p>
                )}
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Total Tickets
                </label>
                <input
                  type="number"
                  placeholder="e.g., 500"
                  {...register("tickets")}
                  className="w-full bg-black/20 border border-white/10 rounded-xl px-4 py-3 text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-[#FF5722]/50 focus:border-[#FF5722] transition"
                />
                {errors.tickets && (
                  <p className="text-red-400 text-sm mt-1.5 flex items-center gap-1"><span className="text-xs">⚠️</span> {errors.tickets.message}</p>
                )}
              </div>

              <div className="md:col-span-2">
                <label className="block text-sm font-medium text-gray-300 mb-2">
                  Event Image (optional)
                </label>
                <div className="w-full border-2 border-dashed border-white/20 rounded-xl px-4 py-6 text-center hover:bg-white/5 transition focus-within:ring-2 focus-within:border-transparent focus-within:ring-[#FF5722]/50">
                    <input
                        type="file"
                        accept="image/*"
                        className="w-full text-sm text-gray-400 file:mr-4 file:py-2.5 file:px-4 file:rounded-lg file:border-0 file:text-sm file:font-semibold file:bg-[#FF5722]/10 file:text-[#FF5722] hover:file:bg-[#FF5722]/20 file:transition file:cursor-pointer cursor-pointer"
                        onChange={(e) => setImage(e.target.files?.[0] || null)}
                    />
                </div>
              </div>
            </div>

            <div className="pt-4">
                <button
                type="submit"
                disabled={isSubmitting || (!isValid && Object.keys(errors).length > 0)}
                className="w-full bg-[#FF5722] hover:bg-[#F4511E] text-white font-bold text-lg py-4 px-6 rounded-xl shadow-[0_0_20px_rgba(255,87,34,0.3)] transition transform hover:-translate-y-1 disabled:opacity-50 disabled:hover:translate-y-0 disabled:shadow-none disabled:cursor-not-allowed flex justify-center items-center gap-3"
                >
                {isSubmitting ? (
                    <>
                        <svg className="animate-spin -ml-1 mr-2 h-5 w-5 text-white" fill="none" viewBox="0 0 24 24">
                            <circle className="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" strokeWidth="4"></circle>
                            <path className="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
                        </svg>
                        Creating Event...
                    </>
                ) : "Launch Event"}
                </button>
            </div>
          </form>
        </div>
      </main>
    </div>
  );
}
