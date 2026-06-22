import { component$, useTask$, useSignal, $ } from '@builder.io/qwik';
import * as echarts from 'echarts';

interface GaugeChartProps {
  value: number;
  max?: number;
  title?: string;
  unit?: string;
  height?: string;
  thresholds?: { value: number; color: string }[];
}

export default component$<GaugeChartProps>(({
  value,
  max = 100,
  title,
  unit = '',
  height = '200px',
  thresholds = [
    { value: 60, color: '#22c55e' },
    { value: 85, color: '#f59e0b' },
    { value: 100, color: '#ef4444' },
  ],
}) => {
  const chartRef = useSignal<HTMLDivElement>();

  const initChart = $(() => {
    if (!chartRef.value) return;

    const chart = echarts.init(chartRef.value);
    
    const colors = thresholds.map(t => t.color);
    const axisLineColors = thresholds.map((t, i) => {
      const prev = i > 0 ? thresholds[i - 1].value : 0;
      return [t.value / max, t.color];
    });

    const option: echarts.EChartsOption = {
      series: [
        {
          type: 'gauge',
          startAngle: 180,
          endAngle: 0,
          min: 0,
          max,
          splitNumber: 4,
          itemStyle: {
            color: value > thresholds[1].value ? colors[2] : value > thresholds[0].value ? colors[1] : colors[0],
          },
          progress: {
            show: true,
            width: 12,
          },
          pointer: {
            show: false,
          },
          axisLine: {
            lineStyle: {
              width: 12,
              color: axisLineColors as any,
            },
          },
          axisTick: {
            show: false,
          },
          splitLine: {
            show: false,
          },
          axisLabel: {
            show: false,
          },
          anchor: {
            show: false,
          },
          title: {
            show: !!title,
            text: title || '',
            offsetCenter: [0, '20%'],
            fontSize: 12,
            color: '#6b7280',
          },
          detail: {
            valueAnimation: true,
            fontSize: 28,
            fontWeight: 'bold',
            offsetCenter: [0, '0%'],
            formatter: `{value}${unit}`,
            color: value > thresholds[1].value ? colors[2] : value > thresholds[0].value ? colors[1] : colors[0],
          },
          data: [{ value }],
        },
      ],
    };

    chart.setOption(option);

    const handleResize = () => chart.resize();
    window.addEventListener('resize', handleResize);

    return () => {
      window.removeEventListener('resize', handleResize);
      chart.dispose();
    };
  });

  useTask$(() => {
    initChart();
  });

  return <div ref={chartRef} style={{ height }}></div>;
});
