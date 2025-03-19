import { Module } from '@nestjs/common';
import { MediafilesDbService } from './mediafiles.db.service';
import { MediafilesService } from './mediafiles.service';

@Module({
  imports: [],
  controllers: [],
  providers: [MediafilesService, MediafilesDbService],
  exports: [MediafilesService],
})
export class MediafilesModule {}
