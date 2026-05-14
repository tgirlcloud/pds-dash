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

  const inviteCodeReq = await client.post("com.atproto.server.createInviteCode", {
    input: {
      useCount: 1,
    },
  })

  console.error(JSON.stringify(inviteCodeReq));
  if (!inviteCodeReq.ok) {
    return {
      error: "Failed to create invite code. Please try again later.",
    };
  }

  const res = await client.post("com.atproto.server.createAccount", {
    input: {
      handle: resolvedHandle,
      email,
      password,
      verificationCode: inviteCodeReq.data.code,
    },
  });
  
  if (res.ok) {
    return { success: "Your account has been created! You can now log in." };
  }

  return {
    error: "Failed to create account. Please try again later.",
  };
}
