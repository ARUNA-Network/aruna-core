<script setup lang="ts">
import { computed } from 'vue'
import type { Block } from '~/types'

const props = defineProps<{
  blocks: Block[]
}>()

const chartPath = computed(() => {
  const points = props.blocks.map(b => b.difficulty).reverse()
  if (points.length < 2) return ''

  const min = Math.min(...points)
  const max = Math.max(...points)
  const range = max - min || 1

  const width = 500
  const height = 130
  const startX = 50
  const startY = 25

  const coords = points.map((p, i) => {
    const x = startX + (i / (points.length - 1)) * width
    const y = startY + height - ((p - min) / range) * height
    return `${x},${y}`
  })

  return `M ${coords.join(' L ')}`
})
</script>

<template>
  <div class="chart-box">
    <div class="chart-header">Block Difficulty Trend</div>
    <svg id="difficulty-chart" viewBox="0 0 600 200" class="svg-chart" aria-label="Line chart showing difficulty over recent blocks">
      <!-- Background grids -->
      <line x1="50" y1="20" x2="550" y2="20" stroke="rgba(220,220,220,0.06)" stroke-width="1" />
      <line x1="50" y1="70" x2="550" y2="70" stroke="rgba(220,220,220,0.06)" stroke-width="1" />
      <line x1="50" y1="120" x2="550" y2="120" stroke="rgba(220,220,220,0.06)" stroke-width="1" />
      <line x1="50" y1="170" x2="550" y2="170" stroke="rgba(220,220,220,0.06)" stroke-width="1" />
      <!-- Axis lines -->
      <line x1="50" y1="170" x2="550" y2="170" stroke="rgba(220,220,220,0.2)" stroke-width="1.5" />
      <line x1="50" y1="20" x2="50" y2="170" stroke="rgba(220,220,220,0.2)" stroke-width="1.5" />
      <!-- Dynamic path -->
      <path :d="chartPath" fill="none" stroke="url(#gradient-difficulty)" stroke-width="3" />
      <!-- Gradients -->
      <defs>
        <linearGradient id="gradient-difficulty" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stop-color="var(--brand-primary)" />
          <stop offset="100%" stop-color="var(--brand-secondary)" />
        </linearGradient>
      </defs>
    </svg>
  </div>
</template>
