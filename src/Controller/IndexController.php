<?php

namespace App\Controller;

use App\Ampere\SystemInfo\OneSecond;
use App\Ampere\SystemInfo\StaticInfo;
use App\Ampere\SystemInfo\ThreeSeconds;
use Symfony\Bundle\FrameworkBundle\Controller\AbstractController;
use Symfony\Component\HttpFoundation\Response;
use Symfony\Component\Routing\Annotation\Route;

class IndexController extends AbstractController
{
    #[Route('/', name: 'index')]
    public function index(StaticInfo $staticInfo, OneSecond $oneSecond, ThreeSeconds $threeSeconds): Response
    {
        $expensive = $threeSeconds->getDto();

        return $this->render('index/index.html.twig', [
            'staticInfo' => $staticInfo->getDto(),
            'liveInfo' => $oneSecond->getDto(),
            'processList' => $expensive->getProcessList(),
            'dockerList' => $expensive->getDockerList(),
            'diskList' => $expensive->getDiskList(),
        ]);
    }

    #[Route('/static-info', name: 'staticInfo')]
    public function staticInfo(StaticInfo $staticInfo): Response
    {
        return $this->render('index/static_info.html.twig', ['info' => $staticInfo->getDto()]);
    }

    #[Route('/dynamic-info', name: 'mutable')]
    public function dynamicInfo(OneSecond $oneSecond, ThreeSeconds $threeSeconds): Response
    {
        $expensive = $threeSeconds->getDto();

        return $this->render('index/dynamic_info.html.twig', [
            'liveInfo' => $oneSecond->getDto(),
            'processList' => $expensive->getProcessList(),
            'diskList' => $expensive->getDiskList()
        ]);
    }
}
