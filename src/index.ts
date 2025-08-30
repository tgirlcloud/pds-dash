import { Client, simpleFetchHandler } from "@atcute/client";
import type { Did } from "@atcute/lexicons";
import { AppBskyActorProfile } from "@atcute/bluesky";

const client = new Client({
  handler: simpleFetchHandler({ service: "https://pds.tgirl.cloud" }),
});

async function getRepos(
  cursor: string | undefined = undefined,
  dids: Did[] = [],
): Promise<Did[]> {
  const response = await client.get("com.atproto.sync.listRepos", {
    params: { cursor },
  });

  if (!response.ok) {
    console.error("failed to get data from pds");
    return [];
  }

  dids.push(...response.data.repos.map((d) => d.did));

  if (response.data.cursor) {
    getRepos(response.data.cursor, dids);
  }

  return dids;
}

// i'm only exporting a subset of the profile data containing only what i need
export interface UserProfile {
  did: Did;
  handle: string;
  displayName?: string;
  avatar?: string;
}

async function getUsersProfile(did: Did): Promise<UserProfile | undefined> {
  const responseRec = await client.get("com.atproto.repo.getRecord", {
    params: {
      repo: did,
      collection: "app.bsky.actor.profile",
      rkey: "self",
    },
    as: "json",
  });

  if (!responseRec.ok) {
    console.error("failed to fetch user profiles");
    return undefined;
  }

  let user = responseRec.data.value as AppBskyActorProfile.Main;

  let avatar: string | undefined = undefined;
  if (user.avatar) {
    // @ts-ignore
    let av = user.avatar.ref["$link"];
    avatar = `https://cdn.bsky.app/img/feed_thumbnail/plain/${did}/${av}`;
  }

  const responseId = await client.get("com.atproto.repo.describeRepo", {
    params: { repo: did },
    as: "json",
  });

  if (!responseId.ok) {
    console.error("failed to resolve identity");
    return undefined;
  }

  return {
    did,
    handle: responseId.data.handle,
    displayName: user.displayName ?? undefined,
    avatar,
  };
}

export async function getPdsUsers() {
  const repos = await getRepos();

  const profiles = [];
  for (const did of repos) {
    const profile = await getUsersProfile(did);
    if (profile) profiles.push(profile);
  }

  return profiles;
}

export async function getVersion(): Promise<string> {
  const response = await fetch("https://pds.tgirl.cloud/xrpc/_health");

  if (!response.ok) {
    console.error("failed to get records count from pds");
    return "0";
  }

  const data = await response.json();

  return data.version;
}
