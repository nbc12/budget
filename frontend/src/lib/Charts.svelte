<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import Chart from 'chart.js/auto';

  // budgetRows: filtered budget rows (only checked categories), each with
  //   category_name, limit_dollars, spent_dollars, category_color, is_income (strings/bools from API)
  // transactions: filtered transactions, each with category_name, category_color, amount_dollars, is_income, notes
  export let budgetRows: any[] = [];
  export let transactions: any[] = [];

  let barCanvas: HTMLCanvasElement;
  let expensePieCanvas: HTMLCanvasElement;
  let incomeTransCanvas: HTMLCanvasElement;
  let expenseTransCanvas: HTMLCanvasElement;
  let barContainer: HTMLDivElement;

  let barChart: Chart | null = null;
  let expensePieChart: Chart | null = null;
  let incomeTransPieChart: Chart | null = null;
  let expenseTransPieChart: Chart | null = null;

  const BAR_THICKNESS = 10;
  const HEIGHT_PER_CATEGORY = BAR_THICKNESS * 6;

  function updateBarHeight(categoryCount: number) {
    const chartPadding = 100;
    const dynamicHeight = Math.max(300, (categoryCount * HEIGHT_PER_CATEGORY) + chartPadding);
    if (barContainer) barContainer.style.height = dynamicHeight + 'px';
  }

  function num(v: any): number {
    const n = parseFloat(v);
    return isNaN(n) ? 0 : n;
  }

  function buildCharts() {
    const rows = budgetRows;
    const labels = rows.map(r => r.category_name);
    const limits = rows.map(r => num(r.limit_dollars));
    const spent = rows.map(r => num(r.spent_dollars));
    const colors = rows.map(r => r.category_color);

    updateBarHeight(labels.length);

    if (barCanvas) {
      barChart = new Chart(barCanvas, {
        type: 'bar',
        data: {
          labels,
          datasets: [
            {
              label: 'Budget',
              data: limits,
              backgroundColor: '#e9ecef',
              borderColor: '#adb5bd',
              borderWidth: 1,
              barThickness: BAR_THICKNESS
            },
            {
              label: 'Actual',
              data: spent,
              backgroundColor: colors,
              borderColor: colors,
              borderWidth: 1,
              barThickness: BAR_THICKNESS
            }
          ]
        },
        options: {
          indexAxis: 'y',
          responsive: true,
          maintainAspectRatio: false,
          plugins: { title: { display: true, text: 'Budget vs Actual' } },
          scales: {
            x: { beginAtZero: true, ticks: { maxTicksLimit: 5 } },
            y: { stacked: false, grid: { display: false } }
          }
        }
      });
    }

    const expenseRows = rows.filter(r => !r.is_income && num(r.spent_dollars) > 0);
    if (expensePieCanvas) {
      expensePieChart = new Chart(expensePieCanvas, {
        type: 'pie',
        data: {
          labels: expenseRows.map(r => r.category_name),
          datasets: [{
            data: expenseRows.map(r => num(r.spent_dollars)),
            backgroundColor: expenseRows.map(r => r.category_color),
            borderWidth: 1
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            title: { display: true, text: 'Expenses by Category' },
            legend: { position: 'bottom', labels: { boxWidth: 10, font: { size: 9 } } }
          }
        }
      });
    }

    const incomeTrans = transactions.filter(t => t.is_income);
    if (incomeTransCanvas) {
      incomeTransPieChart = new Chart(incomeTransCanvas, {
        type: 'pie',
        data: {
          labels: incomeTrans.map(t => `${t.category_name}: ${t.notes || 'Income'}`),
          datasets: [{
            data: incomeTrans.map(t => num(t.amount_dollars)),
            backgroundColor: incomeTrans.map(t => t.category_color),
            borderWidth: 1
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            title: { display: true, text: 'Individual Income Transactions' },
            legend: { display: false }
          }
        }
      });
    }

    const expenseTrans = transactions.filter(t => !t.is_income);
    if (expenseTransCanvas) {
      expenseTransPieChart = new Chart(expenseTransCanvas, {
        type: 'pie',
        data: {
          labels: expenseTrans.map(t => `${t.category_name}: ${t.notes || 'Expense'}`),
          datasets: [{
            data: expenseTrans.map(t => Math.abs(num(t.amount_dollars))),
            backgroundColor: expenseTrans.map(t => t.category_color),
            borderWidth: 1
          }]
        },
        options: {
          responsive: true,
          maintainAspectRatio: false,
          plugins: {
            title: { display: true, text: 'Individual Expense Transactions' },
            legend: { display: false }
          }
        }
      });
    }
  }

  function destroyCharts() {
    barChart?.destroy();
    expensePieChart?.destroy();
    incomeTransPieChart?.destroy();
    expenseTransPieChart?.destroy();
    barChart = expensePieChart = incomeTransPieChart = expenseTransPieChart = null;
  }

  function refreshCharts() {
    if (!(barChart && expensePieChart && incomeTransPieChart && expenseTransPieChart)) {
      buildCharts();
      return;
    }

    updateBarHeight(budgetRows.length);

    barChart.data.labels = budgetRows.map(r => r.category_name);
    barChart.data.datasets[0].data = budgetRows.map(r => num(r.limit_dollars));
    barChart.data.datasets[1].data = budgetRows.map(r => num(r.spent_dollars));
    barChart.data.datasets[1].backgroundColor = budgetRows.map(r => r.category_color) as any;
    barChart.update();

    const expenseRows = budgetRows.filter(r => !r.is_income && num(r.spent_dollars) > 0);
    expensePieChart.data.labels = expenseRows.map(r => r.category_name);
    expensePieChart.data.datasets[0].data = expenseRows.map(r => num(r.spent_dollars));
    expensePieChart.data.datasets[0].backgroundColor = expenseRows.map(r => r.category_color) as any;
    expensePieChart.update();

    const incTrans = transactions.filter(t => t.is_income);
    incomeTransPieChart.data.labels = incTrans.map(t => `${t.category_name}: ${t.notes || 'Income'}`);
    incomeTransPieChart.data.datasets[0].data = incTrans.map(t => num(t.amount_dollars));
    incomeTransPieChart.data.datasets[0].backgroundColor = incTrans.map(t => t.category_color) as any;
    incomeTransPieChart.update();

    const expTrans = transactions.filter(t => !t.is_income);
    expenseTransPieChart.data.labels = expTrans.map(t => `${t.category_name}: ${t.notes || 'Expense'}`);
    expenseTransPieChart.data.datasets[0].data = expTrans.map(t => Math.abs(num(t.amount_dollars)));
    expenseTransPieChart.data.datasets[0].backgroundColor = expTrans.map(t => t.category_color) as any;
    expenseTransPieChart.update();
  }

  onMount(() => {
    buildCharts();
  });

  onDestroy(() => {
    destroyCharts();
  });

  // Re-render whenever the filtered data changes (category filter toggles, month change, refresh)
  $: budgetRows, transactions, (typeof window !== 'undefined' && refreshCharts());
</script>

<div class="row mb-4">
  <div class="col-md-8 mb-4">
    <div class="chart-container" bind:this={barContainer} style="height: auto; background: white; border-radius: 10px; padding: 15px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
      <canvas bind:this={barCanvas}></canvas>
    </div>
  </div>
  <div class="col-md-4 mb-4">
    <div class="chart-container" style="height: 350px; background: white; border-radius: 10px; padding: 15px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
      <canvas bind:this={expensePieCanvas}></canvas>
    </div>
  </div>
</div>

<div class="row mb-4">
  <div class="col-md-6 mb-4">
    <div class="chart-container" style="height: 350px; background: white; border-radius: 10px; padding: 15px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
      <canvas bind:this={incomeTransCanvas}></canvas>
    </div>
  </div>
  <div class="col-md-6 mb-4">
    <div class="chart-container" style="height: 350px; background: white; border-radius: 10px; padding: 15px; box-shadow: 0 4px 6px rgba(0,0,0,0.1);">
      <canvas bind:this={expenseTransCanvas}></canvas>
    </div>
  </div>
</div>
