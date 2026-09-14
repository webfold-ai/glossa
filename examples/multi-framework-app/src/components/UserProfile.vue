<template>
  <div class="user-card">
    <h2>User Profile</h2>
    <p>Logged in as {{ user.name }}. Your subscription is active.</p>
    <button placeholder="Edit profile">Upgrade to <span class="badge">Pro</span></button>
  </div>
</template>

<script>
export default {
  name: 'UserProfile',
  props: ['user']
}
</script>
