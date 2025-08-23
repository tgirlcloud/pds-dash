import { useEffect, useState } from 'react'
import './App.css'
import { getPdsUsers } from '.'
import type { AppBskyActorDefs } from '@atcute/bluesky';

function App() {
  const [users, setUsers] = useState<AppBskyActorDefs.ProfileViewDetailed[] | undefined>();

  useEffect(() => {
    getPdsUsers().then(users => setUsers(users));
  }, [])

  return (
    <div id="layout">
      <aside id="profiles">
        <h1 className="section-title">Profiles</h1>
        <div className="profiles-list">
          {users && users.map((user) => (
            <a className="user-card" key={user.did} href={`https://bsky.app/profile/${user.did}`}>
              {user.banner ? <div className="banner"> <img src={user.banner} alt={`${user.handle} banner`} /></div> : <div className="banner" /> }
              <div className="avatar">
                <img src={user.avatar} alt={`${user.handle} avatar`} />
              </div>
              <div className="user-info">
                <h2>{user.handle}</h2>
                <p className="did">{user.did}</p>
                <p className="description">{user.description}</p>
              </div>
            </a>
          ))}
        </div>
      </aside>

      <main id="info">
        <section id="service-info">
          <h1 className="section-title">Service Information</h1>
          <p>I didn't know what to put here</p>
          <p><strong>Operator:</strong> did:plc:msjw2c6vob56zkr3zx7nt6wc</p>
        </section>

        <section id="join">
          <h1 className="section-title">How to Join</h1>
          <ol>
            <li>First send a dm to @tgirl.cloud</li>
            <li>You should receive a invite code if you are accepted</li>
            <li>You should now use a tool like <a href="https://pdsmoover.com/">pdsmoover</a> to move to the pds</li>
          </ol>
        </section>
      </main>
    </div>
  );
}

export default App
