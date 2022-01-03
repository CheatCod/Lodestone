import {
    ArcElement,
    CategoryScale,
    Chart as ChartJS,
    Legend,
    LineElement,
    LinearScale,
    PointElement,
    Title,
    Tooltip,
} from 'chart.js';
import { Bar, Doughnut, Line } from "react-chartjs-2";
import React, { useEffect, useRef, useState } from "react";

import Card from "./Card";
import { useInterval } from '../utils';

ChartJS.register(
    ArcElement,
    CategoryScale,
    LinearScale,
    PointElement,
    LineElement,
    Title,
    Tooltip,
    Legend
);

function SystemMonitor() {
    const [cpuHistory, setCpuHistory] = useState(new Array(60).fill(0));
    const [ram, setRam] = useState(new Array(60).fill(0));

    const { pollrate, domain, webport } = useContext(ServerContext);

    // return just a number/tuple idfk
    const getCurCpuUsage = async (domain, webport) => {
        // TODO fetch backend for data
        let response = await fetch(`https://${domain}:${webport}/api/sys/`);
        // TODO process data to get a percentage back
    }

    const updateCPU = (domain, webport) => {
        const newCpu = [...cpuHistory];
        newCpu.shift()
        
        const newCpuUsage = getCurCpuUsage(domain, webport);
        newCpu.push(newCpuUsage);
        setCpuHistory(newCpu)
    }

    useInterval(updateCPU(domain, webport), 1000, false)

    const data = {
        labels: [1, 2, 3, 4, 5, 6], // TODO need way to map the 
        datasets: [
            {
                xAxisID: "x",
                yAxisID: "y",
                label: "CPU",
                data: [50, 60, 70, 80, 90, 100],
            },
            {
                xAxisID: "x",
                yAxisID: "y",
                label: "RAM",
                data: [30, 20, 10, 0, 10, 20],
            },
        ],
    }

    console.log(data);

    return (
        <Card>
            <Line
                datasetIdKey="cpu"
                type="line"
                data={data}
                options={{
                    scales: {
                        x: {
                            ticks: {
                                maxTicksLimit: 2,
                                autoSkip: 5,
                            }
                        },
                        y: {
                            beginAtZero: true,
                            min: 0,
                            max: 100,
                            ticks: {
                                stepSize: 25,
                            }

                        }
                    },
                    legend: {
                        display: false
                    },
                    tooltips: {
                        callbacks: {
                            label: function (tooltipItem) {
                                return tooltipItem.yLabel;
                            }
                        }
                    },
                    animation: false
                }}

            />
            <Doughnut
                datasetIdKey="ram"
                data={
                    {
                        labels: ["usage", "empty"],
                        datasets: [
                            {
                                data: [1535, 124]
                            },
                        ],

                    }
                }
            />
        </Card>
    );
}

export default SystemMonitor;