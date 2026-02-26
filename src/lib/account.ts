import "@atcute/atproto";
import { client } from "./fetch";

interface CreateAccountParams {
  handle: string;
  email: string;
  password: string;
    inviteCode?: string;
}

interface CreateAccountResult {
  success?: string;
  error?: string;
}

export async function createAccount({
  handle,
  email,
  password,
}: CreateAccountParams): Promise<CreateAccountResult> {
  const resolvedHandle = (handle.includes(".") ? handle : `${handle}.tgirl.beauty`) as `${string}.${string}`;

  const res = await client.post("com.atproto.server.createAccount", {
    input: {
      handle: resolvedHandle,
      email,
      password
    },
  });
  
  if (res.ok) {
    return { success: "Your account has been created! You can now log in." };
  }

  return {
    error: "Failed to create account. Please try again later.",
  };
}
