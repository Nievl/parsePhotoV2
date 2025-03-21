import { Body, Controller, Delete, Get, Post, Query } from '@nestjs/common';
import { iResult } from '../entities/common';
import { booleanParser } from '../helpers/common';
import { CreateLinkDto, Link } from '../entities/link.entity';
import { LinksService } from './links.service';

@Controller('links')
export class LinksController {
  constructor(private readonly linksService: LinksService) {}

  @Post()
  create(@Body() createLinkDto: CreateLinkDto): Promise<iResult> {
    return this.linksService.createOne(createLinkDto);
  }

  @Get()
  getAll(@Query('isReachable') isReachable: string, @Query('showDuplicate') showDuplicate: string): Promise<Link[]> {
    return this.linksService.getAll(booleanParser(isReachable), booleanParser(showDuplicate));
  }

  @Delete()
  remove(@Query('id') id: string): Promise<iResult> {
    return this.linksService.remove(id);
  }

  @Get('/download')
  public downloadFiles(@Query('id') id: string): Promise<iResult> {
    return this.linksService.downloadFiles(id);
  }

  @Get('/scan_files_for_link')
  public scanFilesForLink(@Query('id') id: string): Promise<iResult> {
    return this.linksService.scanFilesForLink(id);
  }

  @Get('/check_downloaded')
  public checkDownloaded(@Query('id') id: string): Promise<iResult> {
    return this.linksService.checkDownloaded(id);
  }

  @Get('/tag_unreachable')
  public tagUnreachable(@Query('id') id: string, @Query('isReachable') isReachable: string): Promise<iResult> {
    return this.linksService.tagUnreachable(id, booleanParser(isReachable));
  }
}
