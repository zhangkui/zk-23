import { component$, useTask$, useSignal, $ } from '@builder.io/qwik';
import * as echarts from 'echarts';

interface LineChartProps {
  data: { time: string; value: number }[];
  title?: string;
  yAxisName?: string;
  color?: string;
  height?: string;
  smooth?: boolean;
}

export default component$<LineChartProps>(({
  data,
  title,
  yAxisName = '数值',
  color = '#3b82f6',
  height = '300px',
  smooth = true,
}) => {
  const chartRef = useSignal<HTMLDivElement>();
  const chartInstance = useSignal<echarts.ECharts | null>(null);

  const initChart = $(() => {
    if (!chartRef.value) return;
    
    if (chartInstance.value) {
      chartInstance.value.dispose();
    }

    const chart = echarts.init(chartRef.value);
    
    const option: echarts.EChartsOption = {
      title: title ? {
        text: title,
        left: 'center',
        textStyle: { fontSize: 14, fontWeight: 'normal' },
      } : undefined,
      tooltip: {
        trigger: 'axis',
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
        boundaryGap: false,
        data: data.map(d => d.time),
        axisLine: { lineStyle: { color: '#e5e7eb' } },
        axisLabel: { color: '#6b7280', fontSize: 11 },
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
          type: 'line',
          smooth,
          symbol: 'circle',
          symbolSize: 6,
          showSymbol: false,
          lineStyle: { color, width: 2 },
          itemStyle: { color },
          areaStyle: {
            color: new echarts.graphic.LinearGradient(0, 0, 0, 1, [
              { offset: 0, color: color + '30' },
              { offset: 1, color: color + '05' },
            ]),
          },
          data: data.map(d => d.value),
        },
      ],
    };

    chart.setOption(option);
    chartInstance.value = chart;

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
