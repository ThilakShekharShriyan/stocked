"use client";

import { useParams } from "next/navigation";
import { useHouse } from "@/lib/api";
import { PhoneView } from "@/components/PhoneView";

export default function PhonePage() {
  const { person } = useParams<{ person: string }>();
  const { house, error, refresh } = useHouse();
  if (!house) return <p className="p-6 text-stone-600">{error ? `Can’t reach the agent (${error}).` : "Loading…"}</p>;
  return (
    <main className="mx-auto min-h-screen max-w-md bg-stone-100">
      <PhoneView house={house} person={person} onChanged={refresh} />
    </main>
  );
}
