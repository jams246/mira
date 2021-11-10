<?php

namespace App\Command;

use App\Ampere\DockerClient;
use App\Ampere\SystemInfo\ValueObject\DockerProcessValueObject;
use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use Symfony\Component\Cache\Adapter\AdapterInterface;
use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Console\Command\Command;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Output\OutputInterface;

#[AsCommand(
    name: 'ampere:stats:docker',
    description: 'Starts the loop to continuously retrieve docker stats and place the results into memcached',
)]
class AmpereStatsDockerCommand extends Command
{
    public function __construct(private AdapterInterface $cache, private DockerClient $client)
    {
        $this->setProcessTitle('Mira');
        parent::__construct();
    }

    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        /* @phpstan-ignore-next-line */
        while (true) {
            $containerList = $this->client->dispatchCommand('/containers/json');

            $containers = [];
            foreach ($containerList as $container) {
                $containers[] = "/containers/{$container->Id}/stats?stream=false&one-shot=false";
            }

            $containerStats = $this->client->multiCurl($containers);

            $response = new \ArrayObject();
            foreach ($containerList as $key => $container) {
                $cpuDelta = $containerStats[$key]->cpu_stats->cpu_usage->total_usage - $containerStats[$key]->precpu_stats->cpu_usage->total_usage;
                $systemCpuDelta = $containerStats[$key]->cpu_stats->system_cpu_usage - $containerStats[$key]->precpu_stats->system_cpu_usage;
                $numberCpus = $containerStats[$key]->cpu_stats->online_cpus;
                $cpuUsage = (($cpuDelta / $systemCpuDelta) * $numberCpus) * 100.0;

                $usedMemory = $containerStats[$key]->memory_stats->usage - $containerStats[$key]->memory_stats->stats->cache;

                try {
                    $response->append(new DockerProcessValueObject(
                    //remove the first / from all container names
                        \substr($container->Names[0], 1),
                        $container->State,
                        \round($cpuUsage, 1),
                        $usedMemory,
                    ));
                } catch (ValueObjectException $e) {
                    continue;
                }
            }

            $dockerList = $this->cache->getItem('docker.list');
            $dockerList->set($response->getArrayCopy());
            $this->cache->save($dockerList);

            \sleep(1);
        }
    }
}
