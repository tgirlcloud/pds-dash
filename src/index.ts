import type { AppBskyActorDefs } from '@atcute/bluesky';
import { Client, simpleFetchHandler } from '@atcute/client';
import type { ActorIdentifier, Did } from "@atcute/lexicons";

const pdsClient = new Client({
	handler: simpleFetchHandler({ service: "https://pds.tgirl.cloud" }),
});

const bskyClient = new Client({
	handler: simpleFetchHandler({ service: "https://public.api.bsky.app" }),
});

async function getRepos(cursor: string | undefined = undefined, dids: Did[] = []): Promise<Did[]> {
  const response = await pdsClient.get('com.atproto.sync.listRepos', {
    params: { cursor }
  });

  if (!response.ok) {
    console.error("failed to get data from pds");
    return [];
  }

  dids.push(...response.data.repos.map(d => d.did));

  if (response.data.cursor) {
    getRepos(response.data.cursor, dids)
  }

  return dids;
}

async function getUsersProfiles(actors: ActorIdentifier[]): Promise<AppBskyActorDefs.ProfileViewDetailed[]> {
  const response = await bskyClient.get('app.bsky.actor.getProfiles', {
    params: { actors },
  });

  if (!response.ok) {
    console.error("failed to fetch user profiles");
    return [];
  }

  return response.data.profiles;
}

export async function getPdsUsers(): Promise<AppBskyActorDefs.ProfileViewDetailed[]> {
  const repos = await getRepos();
  const profiles = [];
  for (let i = 0; i <= repos.length; i = i + 25) {
    const reposToFetch = repos.slice(i, i + 25);
    const fetchedProfiles = await getUsersProfiles(reposToFetch);
    profiles.push(...fetchedProfiles);
  }

  return profiles;
}
