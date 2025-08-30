import { useEffect, useState } from "react";
import "./App.css";
import { getPdsUsers, getVersion } from ".";
import type { UserProfile } from ".";

function App() {
  const [users, setUsers] = useState<UserProfile[] | undefined>();
  const [version, setVersion] = useState<string | undefined>();

  useEffect(() => {
    getPdsUsers().then((users) => setUsers(users));
    getVersion().then((v) => setVersion(v));
  }, []);

  return (
    <>
      <main className="main">
        <div className="hero card">
          <h1 className="hero-title">tgirl.cloud PDS</h1>
          <p className="hero-subtitle">
            A public personal data server (PDS) for Atproto
          </p>

          <div className="hero-warning">
            <p>
              This PDS is currently invite-only. You’ll need a valid invite code
              to join.
            </p>
          </div>

          <div className="hero-actions">
            <a
              className="btn primary"
              href="mailto:me@isabelroses.com?subject=joining%20the%20PDS"
            >
              Request Invite
            </a>
            <a className="btn" href="https://pdsls.dev/pds.tgirl.cloud">
              Explore
            </a>
          </div>
        </div>

        <div className="stats-cards">
          <div className="stat card">
            <p className="stat-label">Users</p>
            <p className="stat-value">{users?.length ?? "Loading..."}</p>
          </div>

          <div className="stat card">
            <p className="stat-label">PDS Version</p>
            <p className="stat-value">{version}</p>
          </div>
        </div>

        <div>
          <h2 className="section-title">Users</h2>
          <div className="users-grid">
            {users ? (
              users.map((user) => (
                <div key={user.did} className="user-card card">
                  {user.avatar ? (
                    <img
                      src={user.avatar}
                      alt={`${user.displayName ?? user.handle}'s avatar`}
                      className="user-avatar"
                    />
                  ) : (
                    <div className="user-avatar placeholder" />
                  )}
                  <h3 className="user-name">
                    {user.displayName ?? user.handle}
                  </h3>
                  <p className="user-handle">@{user.handle}</p>
                </div>
              ))
            ) : (
              <p>Loading users...</p>
            )}
          </div>
        </div>
      </main>
    </>
  );
}

export default App;
