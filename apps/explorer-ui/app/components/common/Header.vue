<template>
  <nav class="navbar" role="navigation" aria-label="Main navigation">
    <div class="nav-inner">
      <NuxtLink to="/" class="nav-logo" aria-label="ARUNA Explorer Home">
        <div class="logo-icon">⬡</div>
        <div class="logo-text">
          <span class="logo-name">ARUNA</span>
          <span class="logo-sub">Explorer</span>
        </div>
      </NuxtLink>
      <div class="nav-badge">Sumatera Testnet</div>
      
      <ul class="nav-menu" role="menubar">
        <li role="none"><NuxtLink to="/" class="nav-link" active-class="active" role="menuitem">Home</NuxtLink></li>
        <li role="none"><NuxtLink to="/blocks" class="nav-link" active-class="active" role="menuitem">Blocks</NuxtLink></li>
        <li role="none"><NuxtLink to="/transactions" class="nav-link" active-class="active" role="menuitem">Transactions</NuxtLink></li>
        <li role="none"><NuxtLink to="/network" class="nav-link" active-class="active" role="menuitem">Network</NuxtLink></li>
        <li role="none"><NuxtLink to="/validators" class="nav-link" active-class="active" role="menuitem">Validators</NuxtLink></li>
        <li role="none"><NuxtLink to="/stats" class="nav-link" active-class="active" role="menuitem">Supply</NuxtLink></li>
      </ul>
      <slot name="search" />
    </div>
  </nav>
</template>

<style scoped>
/* Scoped overrides if needed, global navbar styles are in main.css */
</style>
