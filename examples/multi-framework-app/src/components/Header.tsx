import React from 'react';

export function Header({ userName, notificationCount }: { userName: string; notificationCount: number }) {
  return (
    <header className="app-header">
      <div className="logo">GlobalApp</div>
      <nav>
        <p>Welcome back, {userName}! You have {notificationCount} notifications.</p>
        <input type="text" placeholder="Search resources" />
        <button>Visit <a href="/settings">Settings</a></button>
      </nav>
    </header>
  );
}
