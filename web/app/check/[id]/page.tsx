import type { Metadata } from "next";
import { CheckRoom } from "@/components/CheckRoom";

export const metadata: Metadata = {
  title: "Evidence room — StillTrue",
};

export default async function CheckPage({
  params,
}: {
  params: Promise<{ id: string }>;
}) {
  const { id } = await params;
  return <CheckRoom id={id} />;
}
