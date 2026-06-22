import { component$, useTask$, useSignal, $ } from '@builder.io/qwik';
import * as echarts from 'echarts';

interface BarChartProps {
  categories: string[];
  values: number[];
  title?: string;
  yAxisName?: string;
  color?: string;
  height?: string;
}

export default component$<BarChartProps>(({
  categories,
  values,
  title,
  yAxisName = '数量',
  color = '#3b82f6',
  height = '300px',
}) => {
  const chartRef = useSignal<HTMLDivElement>();

  const initChart = $(() => {
    if (!chartRef.value) return;

    const chart = echarts.init(chartRef.value);
    
    const option: echarts.EChartsOption = {
      title: title ? {
        text: title,
        left: 'center',
        textStyle: { fontSize: 14, fontWeight: 'normal' },
      } : undefined,
      tooltip: {
        trigger: 'axis',
        axisPointer: { type: 'shadow' },
        backgroundColor: 'rgba(255, 255, 255, 0.95)',
        borderColor: '#e5e7eb',
        borderWidth: 1,
        textStyle: { color: '#374151' },
      },
      grid: {
        left: '3%',
        right: '4%',
        bottom: '3%',
        top: title ? '15%' : '3%',
        containLabel: true,
      },
      xAxis: {
        type: 'category',
        data: categories,
        axisLine: { lineStyle: { color: '#e5e7eb' } },
        axisLabel: { color: '#6b7280', fontSize: 11, rotate: 30 },
      },
      yAxis: {
        type: 'value',
        name: yAxisName,
        nameTextStyle: { color: '#6b7280', fontSize: 11 },
        axisLine: { lineStyle: { color: '#e5e7eb' } },
        axisLabel: { color: '#6b7280', fontSize: 11 },
        splitLine: { lineStyle: { color: '#f3f4f6' } },
      },
      series: [
        {
          type: 'bar',
          data: values,
          barWidth: '50%',
          itemStyle: {
            color,
            borderRadius: [4, 4, 0, 0],
          },
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
