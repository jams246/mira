<?php

namespace App\Command;

use App\Ampere\DockerClient;
use App\Ampere\DockerClient\Endpoint\ContainerList;
use App\Ampere\DockerClient\Endpoint\ContainerStats;
use App\Ampere\DockerClient\Request\Context;
use App\Ampere\DockerClient\Response\Response;
use App\Ampere\DockerClient\ValueObject\ContainerIdValueObject;
use App\Ampere\SystemInfo\ValueObject\DockerProcessValueObject;
use App\Ampere\SystemInfo\ValueObject\Exception\ValueObjectException;
use Psr\Cache\CacheItemPoolInterface;
use Psr\Cache\InvalidArgumentException;
use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Console\Command\Command;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Output\OutputInterface;

#[AsCommand(
    name: 'ampere:stats:docker',
    description: 'Starts a loop to continuously retrieve docker stats and place the results into memcached',
)]
class AmpereStatsDockerCommand extends Command
{
    private const DOCKER_SOCK_PATH = '/var/run/docker.sock';

    public function __construct(private CacheItemPoolInterface $cache)
    {
        $this->setProcessTitle('Mira');
        parent::__construct();
    }

    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        try {
            $this->cache->deleteItem('docker.list');
        } catch (\Exception|InvalidArgumentException $e) {
        } finally {
            if (!\file_exists(self::DOCKER_SOCK_PATH)) {
                return 0;
            }
        }

        $containerStreams = [];
        $containerListContext = Context::create(new ContainerList());
        while (true) {
            $containerListConn = DockerClient\SocketConnection::create($containerListContext);
            $response = $containerListConn->request();
            $containerListConn->closeConn();

            $containerList = $response->getContent();

            foreach ($containerList as $container) {
                if (isset($containerStreams[$container['Id']])) {
                    continue;
                }
                $context = Context::create(new ContainerStats(new ContainerIdValueObject($container['Id'])));
                $conn = DockerClient\SocketConnection::create($context);
                $conn->startStream();
                $containerStreams[$container['Id']] = $conn->getConn();
            }
            \sleep(1);

            $response = new \ArrayObject();
            foreach ($containerList as $container) {
                $containerStats = (new Response(\stream_get_contents($containerStreams[$container['Id']], -1)))->getContent();

                try {
                    $cpuDelta = $containerStats['cpu_stats']['cpu_usage']['total_usage'] - $containerStats['precpu_stats']['cpu_usage']['total_usage'];
                    $systemCpuDelta = $containerStats['cpu_stats']['system_cpu_usage'] - $containerStats['precpu_stats']['system_cpu_usage'];
                    $numberCpus = $containerStats['cpu_stats']['online_cpus'];
                    $cpuUsage = (($cpuDelta / $systemCpuDelta) * $numberCpus) * 100.0;

                    $usedMemory = $containerStats['memory_stats']['usage'] - $containerStats['memory_stats']['stats']['cache'];

                    $response->append(new DockerProcessValueObject(
                    //remove the first / from all container names
                        \substr($container['Names'][0], 1),
                        $container['State'],
                        \round($cpuUsage, 1),
                        $usedMemory,
                    ));
                } catch (ValueObjectException|\Exception $e) {
                    continue;
                }
            }

            $dockerList = $this->cache->getItem('docker.list');
            $dockerList->set($response->getArrayCopy());
            $this->cache->save($dockerList);
        }
    }
}
