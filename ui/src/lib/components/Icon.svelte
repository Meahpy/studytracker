<script lang="ts">
  import { icons } from '../icons';

  interface Props {
    name: string;
    size?: number;
    stroke?: number;
    class?: string;
  }

  let { name, size = 20, stroke = 1.75, class: className = '' }: Props = $props();
  const markup = $derived(icons[name] ?? icons.more);
</script>

<svg
  class={className}
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke="currentColor"
  stroke-width={stroke}
  stroke-linecap="round"
  stroke-linejoin="round"
  aria-hidden="true"
  focusable="false"
>
  {@html markup}
</svg>
