<script setup lang="ts">
import { useRoute } from '#app'
import Header from '~/components/common/Header.vue'
import Footer from '~/components/common/Footer.vue'
import SearchBar from '~/components/common/SearchBar.vue'

const route = useRoute()
</script>

<template>
  <div class="app-layout">
    <!-- Header with conditional navbar SearchBar -->
    <Header>
      <template #search v-if="route.path !== '/'">
        <SearchBar :small="true" placeholder="Search..." />
      </template>
    </Header>

    <!-- Page Content Wrapper -->
    <div class="page-content">
      <slot />
    </div>

    <!-- Footer -->
    <Footer />
  </div>
</template>

<style scoped>
.app-layout {
  display: flex;
  flex-direction: column;
  min-height: 100vh;
}

.page-content {
  flex: 1;
}
</style>
