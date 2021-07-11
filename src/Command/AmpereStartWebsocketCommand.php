<?php

namespace App\Command;

use App\Ampere\SystemInfo\OneSecond;
use App\Ampere\SystemInfo\ThreeSeconds;
use App\Websocket\LiveTopic;
use Ratchet\Http\HttpServer;
use Ratchet\Http\Router;
use Ratchet\Server\IoServer;
use Ratchet\WebSocket\WsServer;
use React\EventLoop\Factory as LoopFactory;
use React\Socket\Server;
use Symfony\Component\Console\Attribute\AsCommand;
use Symfony\Component\Console\Command\Command;
use Symfony\Component\Console\Input\InputInterface;
use Symfony\Component\Console\Output\OutputInterface;
use Symfony\Component\Routing\Matcher\UrlMatcher;
use Symfony\Component\Routing\RequestContext;
use Symfony\Component\Routing\Route;
use Symfony\Component\Routing\RouteCollection;
use Symfony\Component\Serializer\SerializerInterface;

#[AsCommand(
    name: 'ampere:start:websocket',
    description: 'Starts the websocket server based on Ratchet.io',
)]
class AmpereStartWebsocketCommand extends Command
{
    private const WS_PORT = 44357;

    public function __construct(private SerializerInterface $serializer, private OneSecond $oneSecond, private ThreeSeconds $threeSeconds)
    {
        $this->setProcessTitle('Mira');
        parent::__construct();
    }

    /**
     * @url https://gist.github.com/rahulhaque/2a92380e54779d03660e01da3851d24d
     */
    protected function execute(InputInterface $input, OutputInterface $output): int
    {
        $loop = LoopFactory::create();
        $socket = new Server('0.0.0.0:'.self::WS_PORT, $loop);
        $routes = new RouteCollection();

        $liveTopic = new LiveTopic($output);
        $liveInfoServer = new WsServer($liveTopic);
        $liveInfoServer->enableKeepAlive($loop);
        $routes->add('liveInfo', new Route('/live.info', [
            '_controller' => $liveInfoServer,
            'allowedOrigins' => '*',
        ]));

        $urlMatcher = new UrlMatcher($routes, new RequestContext());
        $router = new Router($urlMatcher);
        $server = new IoServer(
            new HttpServer($router),
            $socket,
            $loop
        );

        $loop->addPeriodicTimer(1, function () use ($liveTopic) {
            if ($liveTopic->getClientCount() >= 1) {
                $data = [
                    'event' => (new \ReflectionClass($this->oneSecond))->getShortName(),
                    'data' => $this->oneSecond->getDto(),
                ];

                $liveTopic->sendLiveSystemData($this->serializer->serialize($data, 'json'));
            }
        });

        $loop->addPeriodicTimer(3, function () use ($liveTopic) {
            if ($liveTopic->getClientCount() >= 1) {
                $data = [
                    'event' => (new \ReflectionClass($this->threeSeconds))->getShortName(),
                    'data' => $this->threeSeconds->getDto(),
                ];

                $liveTopic->sendLiveSystemData($this->serializer->serialize($data, 'json'));
            }
        });

        $output->writeln('Websocket server started on: 0.0.0.0:'.self::WS_PORT);

        $server->run();

        return 0;
    }
}
